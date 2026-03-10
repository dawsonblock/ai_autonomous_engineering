from __future__ import annotations

import ast
from pathlib import Path
from typing import Dict, List


class BehaviorCfgBuilder:
    def build(self, repo_path: str, file_path: str, function_name: str) -> Dict[str, object]:
        target = Path(repo_path) / file_path
        if not target.exists():
            return {"cfg_nodes": 0, "branches": [], "loops": [], "returns": [], "assignments": []}
        tree = ast.parse(target.read_text(encoding="utf-8"), filename=str(target))
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == function_name:
                return {
                    "cfg_nodes": sum(1 for _ in ast.walk(node)),
                    "branches": self._line_numbers(node, (ast.If, ast.Try, ast.BoolOp)),
                    "loops": self._line_numbers(node, (ast.For, ast.While)),
                    "returns": self._line_numbers(node, (ast.Return,)),
                    "assignments": self._line_numbers(node, (ast.Assign, ast.AnnAssign, ast.AugAssign)),
                }
        return {"cfg_nodes": 0, "branches": [], "loops": [], "returns": [], "assignments": []}

    def _line_numbers(self, node: ast.AST, kinds: tuple[type, ...]) -> List[int]:
        numbers = []
        for child in ast.walk(node):
            if isinstance(child, kinds):
                line = getattr(child, "lineno", 0)
                if line:
                    numbers.append(line)
        return sorted(set(numbers))
