from __future__ import annotations

import re
import shutil
import tempfile
from pathlib import Path

from aae.contracts.sandbox import SandboxRunResult, SandboxRunSpec
from aae.patching.git_ops.multi_file_editor import MultiFileEditor


class ArtifactCollector:
    def __init__(self, multi_file_editor: MultiFileEditor | None = None) -> None:
        self.multi_file_editor = multi_file_editor or MultiFileEditor()

    def prepare_workspace(self, spec: SandboxRunSpec, container_id: str) -> tuple[str, str, dict]:
        artifact_root = Path(spec.artifact_dir) if spec.artifact_dir else Path(tempfile.mkdtemp(prefix="aae-sandbox-"))
        artifact_root.mkdir(parents=True, exist_ok=True)
        workspace = artifact_root / "workspace"
        if workspace.exists():
            shutil.rmtree(workspace)
        shutil.copytree(
            spec.repo_path,
            workspace,
            ignore=shutil.ignore_patterns(".git", "__pycache__", ".pytest_cache", ".venv", ".sandbox_artifacts"),
        )
        patch_metadata = {
            "patch_apply_status": "skipped",
            "rollback_status": "",
            "editable_workspace": str(workspace),
            "counterexample_paths": list(spec.ephemeral_test_paths),
            "patch_apply_details": {},
        }
        diffs = [diff for diff in ([spec.patch_diff] + list(spec.patch_bundle)) if diff]
        if diffs:
            result = self.multi_file_editor.apply_multi_file_patch(str(workspace), diffs)
            patch_metadata["patch_apply_details"] = result
            if result["applied"]:
                patch_metadata["patch_apply_status"] = "applied-git"
            else:
                patch_metadata["rollback_status"] = "rolled_back" if result.get("rollback", {}).get("rolled_back") else "rollback-failed"
                if len(diffs) == 1:
                    self._apply_unified_diff(workspace, diffs[0])
                    patch_metadata["patch_apply_status"] = "applied-fallback"
                else:
                    patch_metadata["patch_apply_status"] = "apply-failed"
        trace_path = artifact_root / "trace_records.jsonl"
        return str(workspace), str(trace_path), patch_metadata

    def collect(self, result: SandboxRunResult) -> SandboxRunResult:
        workspace = Path(result.applied_workspace)
        artifact_paths = set(result.artifact_paths)
        trace_paths = set(result.trace_paths)
        coverage_path = result.coverage_path
        if workspace.exists():
            for candidate in workspace.rglob("*"):
                if not candidate.is_file():
                    continue
                if candidate.name in {"coverage.xml", "junit.xml", ".coverage"}:
                    artifact_paths.add(str(candidate))
                    if candidate.name in {"coverage.xml", ".coverage"} and not coverage_path:
                        coverage_path = str(candidate)
                if candidate.name.endswith(".jsonl") and "trace" in candidate.name:
                    trace_paths.add(str(candidate))
                    artifact_paths.add(str(candidate))
        return result.model_copy(
            update={
                "artifact_paths": sorted(artifact_paths),
                "trace_paths": sorted(trace_paths),
                "coverage_path": coverage_path,
            }
        )

    def _apply_unified_diff(self, workspace: Path, diff_text: str) -> None:
        lines = diff_text.splitlines()
        index = 0
        while index < len(lines):
            if not lines[index].startswith("--- "):
                index += 1
                continue
            old_path = _diff_path(lines[index][4:])
            index += 1
            if index >= len(lines) or not lines[index].startswith("+++ "):
                break
            new_path = _diff_path(lines[index][4:])
            target_path = workspace / (new_path or old_path)
            target_path.parent.mkdir(parents=True, exist_ok=True)
            original_text = target_path.read_text(encoding="utf-8") if target_path.exists() else ""
            original_lines = original_text.splitlines()
            updated_lines = []
            original_index = 0
            index += 1
            while index < len(lines) and not lines[index].startswith("--- "):
                line = lines[index]
                if not line.startswith("@@"):
                    index += 1
                    continue
                match = re.match(r"@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@", line)
                if not match:
                    index += 1
                    continue
                old_start = int(match.group(1))
                updated_lines.extend(original_lines[original_index : old_start - 1])
                original_index = old_start - 1
                index += 1
                while index < len(lines) and not lines[index].startswith(("@@", "--- ")):
                    hunk_line = lines[index]
                    prefix = hunk_line[:1]
                    content = hunk_line[1:]
                    if prefix == " ":
                        updated_lines.append(content)
                        original_index += 1
                    elif prefix == "-":
                        original_index += 1
                    elif prefix == "+":
                        updated_lines.append(content)
                    index += 1
            updated_lines.extend(original_lines[original_index:])
            trailing_newline = original_text.endswith("\n") or any(line.startswith("+") for line in lines)
            target_path.write_text("\n".join(updated_lines) + ("\n" if trailing_newline else ""), encoding="utf-8")


def _diff_path(value: str) -> str:
    value = value.strip()
    if value.startswith("a/") or value.startswith("b/"):
        return value[2:]
    return value
