from __future__ import annotations

import subprocess
from pathlib import Path


class GitSafety:
    def ensure_git_snapshot(self, repo_path: str) -> dict:
        path = Path(repo_path)
        created_repo = False
        if not (path / ".git").exists():
            created_repo = True
            self._run(["git", "init", "-q"], repo_path)
            self._run(["git", "checkout", "-B", "aae-sandbox"], repo_path)
            self._run(["git", "config", "user.email", "aae@local.invalid"], repo_path)
            self._run(["git", "config", "user.name", "AAE Sandbox"], repo_path)
            self._run(["git", "add", "-A"], repo_path)
            self._run(["git", "commit", "-qm", "aae baseline"], repo_path)
        self.ensure_clean_repo(repo_path)
        return {"created_repo": created_repo, "repo_path": str(path)}

    def ensure_clean_repo(self, repo_path: str) -> None:
        status = self._run(["git", "status", "--porcelain"], repo_path, capture_output=True)
        if status.stdout.strip():
            raise RuntimeError("repository snapshot is not clean")

    def changed_files(self, repo_path: str) -> list[str]:
        result = self._run(["git", "diff", "--name-only"], repo_path, capture_output=True)
        return [line.strip() for line in result.stdout.splitlines() if line.strip()]

    def _run(self, args: list[str], repo_path: str, capture_output: bool = False):
        return subprocess.run(
            args,
            cwd=repo_path,
            check=True,
            text=True,
            capture_output=capture_output,
        )
