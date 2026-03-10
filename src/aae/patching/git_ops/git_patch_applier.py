from __future__ import annotations

import subprocess

from aae.patching.git_ops.git_safety import GitSafety


class GitPatchApplier:
    def __init__(self, git_safety: GitSafety | None = None) -> None:
        self.git_safety = git_safety or GitSafety()

    def apply_patch(self, repo_path: str, diff: str) -> dict:
        self.git_safety.ensure_clean_repo(repo_path)
        process = subprocess.run(
            ["git", "apply", "--whitespace=fix"],
            input=diff.encode("utf-8"),
            cwd=repo_path,
            capture_output=True,
        )
        if process.returncode != 0:
            return {
                "applied": False,
                "conflict": True,
                "stdout": process.stdout.decode("utf-8", "ignore"),
                "stderr": process.stderr.decode("utf-8", "ignore"),
                "changed_files": [],
            }
        changed_files = self.git_safety.changed_files(repo_path)
        return {
            "applied": True,
            "conflict": False,
            "stdout": process.stdout.decode("utf-8", "ignore"),
            "stderr": process.stderr.decode("utf-8", "ignore"),
            "changed_files": changed_files,
        }
