from __future__ import annotations

from pathlib import Path


class ImageBuilder:
    def choose_image(self, repo_path: str) -> str:
        root = Path(repo_path)
        if any(root.rglob("pyproject.toml")) or any(root.rglob("*.py")):
            return "python:3.11-slim"
        return "alpine:3.20"
