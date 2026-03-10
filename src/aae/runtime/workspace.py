from __future__ import annotations

import asyncio
import json
import shutil
import subprocess
from pathlib import Path

from aae.contracts.graph import RepoWorkspace


class RepoMaterializer:
    def __init__(self, artifacts_dir: str = ".artifacts") -> None:
        self.artifacts_dir = Path(artifacts_dir)

    async def materialize(
        self,
        workflow_id: str,
        repo_url: str | None = None,
        repo_path: str | None = None,
    ) -> RepoWorkspace:
        source = repo_path or repo_url or ""
        if not source:
            raise ValueError("repo_url or repo_path is required to materialize a workspace")

        workspace_dir = self.artifacts_dir / "workspaces" / workflow_id
        repo_target = workspace_dir / "repo"
        metadata_path = workspace_dir / "metadata.json"
        if metadata_path.exists() and repo_target.exists():
            return RepoWorkspace.model_validate(json.loads(metadata_path.read_text(encoding="utf-8")))

        workspace_dir.mkdir(parents=True, exist_ok=True)
        local_source = _local_source_path(source)
        if local_source is not None:
            resolved = local_source.resolve()
            if resolved != repo_target.resolve():
                await asyncio.to_thread(_copy_repo_tree, resolved, repo_target)
            checkout_ref = _git_rev_parse(repo_target)
        else:
            await _git_clone(source, repo_target)
            checkout_ref = _git_rev_parse(repo_target)

        workspace = RepoWorkspace(
            workflow_id=workflow_id,
            source=source,
            repo_path=str(repo_target.resolve()),
            artifacts_dir=str(workspace_dir.resolve()),
            checkout_ref=checkout_ref,
        )
        metadata_path.write_text(
            json.dumps(workspace.model_dump(mode="json"), indent=2, sort_keys=True),
            encoding="utf-8",
        )
        return workspace


def _local_source_path(source: str) -> Path | None:
    if source.startswith("file://"):
        return Path(source[7:])
    path = Path(source)
    if path.exists():
        return path
    return None


def _copy_repo_tree(source: Path, destination: Path) -> None:
    if destination.exists():
        shutil.rmtree(destination)
    shutil.copytree(source, destination, dirs_exist_ok=False, ignore=shutil.ignore_patterns(".git", "__pycache__", ".pytest_cache"))


async def _git_clone(source: str, destination: Path) -> None:
    if destination.exists():
        shutil.rmtree(destination)
    process = await asyncio.create_subprocess_exec(
        "git",
        "clone",
        "--depth",
        "1",
        source,
        str(destination),
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    _, stderr = await process.communicate()
    if process.returncode != 0:
        raise RuntimeError("git clone failed: %s" % stderr.decode("utf-8", "ignore").strip())


def _git_rev_parse(repo_path: Path) -> str | None:
    if not (repo_path / ".git").exists():
        return None
    try:
        return (
            subprocess.check_output(
                ["git", "-C", str(repo_path), "rev-parse", "HEAD"],
                stderr=subprocess.DEVNULL,
                text=True,
            ).strip()
            or None
        )
    except Exception:
        return None
