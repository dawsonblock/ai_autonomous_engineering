from __future__ import annotations

from typing import Dict, List, Set


class DependencyGraph:
    def __init__(self) -> None:
        self.imports: Dict[str, Set[str]] = {}
        self.reverse_imports: Dict[str, Set[str]] = {}

    def add_import(self, source_file: str, imported_module: str) -> None:
        self.imports.setdefault(source_file, set()).add(imported_module)
        self.reverse_imports.setdefault(imported_module, set()).add(source_file)

    def dependencies_of(self, file_path: str) -> Set[str]:
        return self.imports.get(file_path, set())

    def dependents_of(self, module: str) -> Set[str]:
        return self.reverse_imports.get(module, set())

    def transitive_dependents(self, module: str) -> Set[str]:
        visited: Set[str] = set()
        stack = [module]
        while stack:
            current = stack.pop()
            if current in visited:
                continue
            visited.add(current)
            stack.extend(self.reverse_imports.get(current, set()))
        visited.discard(module)
        return visited

    @property
    def module_count(self) -> int:
        return len(set(self.imports.keys()) | set(self.reverse_imports.keys()))
