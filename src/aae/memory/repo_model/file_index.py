from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Set


@dataclass
class FileEntry:
    path: str
    language: str = ""
    size_bytes: int = 0
    symbols: List[str] = field(default_factory=list)
    imports: List[str] = field(default_factory=list)


_EXTENSION_LANGUAGE = {
    ".py": "python",
    ".js": "javascript",
    ".ts": "typescript",
    ".tsx": "typescript",
    ".jsx": "javascript",
    ".java": "java",
    ".go": "go",
    ".rs": "rust",
    ".rb": "ruby",
    ".c": "c",
    ".cpp": "cpp",
    ".h": "c",
}

_IGNORED_DIRS: Set[str] = {
    ".git",
    "__pycache__",
    ".venv",
    "node_modules",
    ".artifacts",
    ".pytest_cache",
}


class FileIndex:
    def __init__(self) -> None:
        self.files: Dict[str, FileEntry] = {}

    def scan(self, repo_path: str) -> None:
        root = Path(repo_path).resolve()
        for file_path in sorted(root.rglob("*")):
            if not file_path.is_file():
                continue
            if any(part in _IGNORED_DIRS for part in file_path.relative_to(root).parts):
                continue
            if file_path.name.startswith("."):
                continue
            relative = file_path.relative_to(root).as_posix()
            lang = _EXTENSION_LANGUAGE.get(file_path.suffix, "")
            try:
                size = file_path.stat().st_size
            except OSError:
                size = 0
            self.files[relative] = FileEntry(
                path=relative,
                language=lang,
                size_bytes=size,
            )

    def get(self, path: str) -> FileEntry | None:
        return self.files.get(path)

    def by_language(self, language: str) -> List[FileEntry]:
        return [entry for entry in self.files.values() if entry.language == language]

    @property
    def count(self) -> int:
        return len(self.files)
