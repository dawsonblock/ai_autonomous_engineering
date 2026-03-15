from __future__ import annotations

from typing import Any, Dict, List, Set

from aae.memory.repo_model.dependency_graph import DependencyGraph
from aae.memory.repo_model.file_index import FileIndex
from aae.memory.repo_model.symbol_graph import SymbolGraph


class RepoModel:
    def __init__(self) -> None:
        self.files = FileIndex()
        self.symbols = SymbolGraph()
        self.dependencies = DependencyGraph()

    def update_from_repo(self, path: str) -> Dict[str, Any]:
        self.files.scan(path)
        return {
            "file_count": self.files.count,
        }

    def impacted_modules(self, changed_file: str) -> Set[str]:
        dependents = self.dependencies.transitive_dependents(changed_file)
        return dependents

    def impacted_tests(self, changed_file: str) -> List[str]:
        dependents = self.impacted_modules(changed_file)
        test_files: List[str] = []
        for dep in dependents:
            entry = self.files.get(dep)
            if entry and ("test_" in entry.path or "_test." in entry.path):
                test_files.append(dep)
        changed_entry = self.files.get(changed_file)
        if changed_entry and ("test_" in changed_entry.path or "_test." in changed_entry.path):
            test_files.append(changed_file)
        return sorted(set(test_files))

    def summary(self) -> Dict[str, Any]:
        return {
            "files": self.files.count,
            "symbols": len(self.symbols.symbols),
            "dependencies": self.dependencies.module_count,
        }
