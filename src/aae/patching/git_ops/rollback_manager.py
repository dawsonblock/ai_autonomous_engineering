from __future__ import annotations

import subprocess


class RollbackManager:
    def rollback(self, repo_path: str) -> dict:
        process = subprocess.run(
            ["git", "reset", "--hard", "HEAD"],
            cwd=repo_path,
            capture_output=True,
            text=True,
        )
        return {
            "rolled_back": process.returncode == 0,
            "stdout": process.stdout,
            "stderr": process.stderr,
        }
