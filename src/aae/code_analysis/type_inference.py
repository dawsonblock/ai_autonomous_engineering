from __future__ import annotations

import ast
from pathlib import Path


class TypeInferenceEngine:
    def infer_for_function(self, repo_path: str, file_path: str, function_name: str) -> dict[str, str]:
        target_file = Path(repo_path) / file_path
        tree = ast.parse(target_file.read_text(encoding="utf-8"), filename=str(target_file))
        inferred: dict[str, str] = {}
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == function_name:
                for arg in node.args.args:
                    if arg.annotation is not None:
                        inferred[arg.arg] = self._annotation_name(arg.annotation)
                for child in ast.walk(node):
                    if isinstance(child, ast.Assign) and len(child.targets) == 1 and isinstance(child.targets[0], ast.Name):
                        inferred[child.targets[0].id] = self._value_type(child.value)
                return inferred
        return inferred

    def _annotation_name(self, node: ast.AST) -> str:
        if isinstance(node, ast.Name):
            return node.id
        if isinstance(node, ast.Subscript):
            return self._annotation_name(node.value)
        if isinstance(node, ast.Attribute):
            return node.attr
        return "unknown"

    def _value_type(self, node: ast.AST) -> str:
        if isinstance(node, ast.Constant):
            return type(node.value).__name__
        if isinstance(node, ast.Dict):
            return "dict"
        if isinstance(node, ast.List):
            return "list"
        if isinstance(node, ast.Set):
            return "set"
        if isinstance(node, ast.Tuple):
            return "tuple"
        if isinstance(node, ast.Call):
            if isinstance(node.func, ast.Name):
                return node.func.id
            if isinstance(node.func, ast.Attribute):
                return node.func.attr
        return "unknown"
