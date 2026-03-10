"""Diff-aware file analysis for PR-mode scanning."""

from __future__ import annotations

import subprocess
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class DiffAnalysis:
    """Result of analyzing git diff between base and head."""

    changed_files: list[str] = field(default_factory=list)
    blast_radius_files: list[str] = field(default_factory=list)
    all_relevant_files: list[str] = field(default_factory=list)
    base_sha: str = ""
    head_sha: str = "HEAD"

    @property
    def file_count(self) -> int:
        return len(self.all_relevant_files)


def analyze_diff(repo_path: str, base_sha: str, head_sha: str = "HEAD") -> DiffAnalysis:
    """Analyze git diff to find changed files and their blast radius.

    Blast radius = files that import from or are imported by changed files.
    """

    repo = Path(repo_path)

    try:
        result = subprocess.run(
            ["git", "diff", "--name-only", "--diff-filter=ACMR", base_sha, head_sha],
            cwd=str(repo),
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
    except (subprocess.SubprocessError, OSError):
        return DiffAnalysis(base_sha=base_sha, head_sha=head_sha)

    changed = [line for line in result.stdout.splitlines() if line and _is_scannable(line)]
    if not changed:
        return DiffAnalysis(base_sha=base_sha, head_sha=head_sha)

    blast_radius: set[str] = set()
    for changed_file in changed:
        module_name = _file_to_module(changed_file)
        if not module_name:
            continue

        try:
            grep_result = subprocess.run(
                [
                    "git",
                    "grep",
                    "-l",
                    module_name,
                    head_sha,
                    "--",
                    "*.py",
                    "*.ts",
                    "*.js",
                    "*.go",
                    "*.java",
                    "*.rb",
                ],
                cwd=str(repo),
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            for line in grep_result.stdout.splitlines():
                if not line:
                    continue
                file_path = line.split(":", 1)[1] if ":" in line else line
                if file_path not in changed and _is_scannable(file_path):
                    blast_radius.add(file_path)
        except (subprocess.SubprocessError, OSError):
            continue

    all_relevant = sorted(set(changed) | blast_radius)
    return DiffAnalysis(
        changed_files=sorted(changed),
        blast_radius_files=sorted(blast_radius),
        all_relevant_files=all_relevant,
        base_sha=base_sha,
        head_sha=head_sha,
    )


def _is_scannable(file_path: str) -> bool:
    """Check if a file should be included in security scanning."""

    skip_dirs = ("tests/", "test/", "vendor/", "node_modules/", ".git/", "__pycache__/")
    skip_extensions = (".md", ".txt", ".yml", ".yaml", ".json", ".toml", ".cfg", ".ini", ".lock")
    if any(file_path.startswith(directory) for directory in skip_dirs):
        return False
    if any(file_path.endswith(extension) for extension in skip_extensions):
        return False
    return True


def _file_to_module(file_path: str) -> str:
    """Convert file path to importable module name for blast radius search."""

    if file_path.endswith(".py"):
        return file_path.replace("/", ".").removesuffix(".py").split(".")[-1]
    if file_path.endswith((".ts", ".js")):
        return file_path.split("/")[-1].removesuffix(".ts").removesuffix(".js")
    return file_path.split("/")[-1].split(".")[0]
