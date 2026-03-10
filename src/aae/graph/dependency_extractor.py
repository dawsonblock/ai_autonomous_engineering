from __future__ import annotations

from pathlib import Path


def module_name_from_path(root_path: Path, file_path: Path) -> str:
    relative = file_path.relative_to(root_path).with_suffix("")
    parts = list(relative.parts)
    if parts and parts[-1] == "__init__":
        parts = parts[:-1]
    return ".".join(parts)


def normalize_import_name(name: str) -> str:
    return name.strip(".")


def is_python_test_path(file_path: Path) -> bool:
    name = file_path.name
    return "tests" in file_path.parts or name.startswith("test_") or name.endswith("_test.py")
