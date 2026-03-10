from __future__ import annotations

import ast
from pathlib import Path

from aae.contracts.graph import SemanticSummary


BRANCH_NODE_TYPES = [ast.If, ast.For, ast.While, ast.Try, ast.BoolOp]
if hasattr(ast, "Match"):
    BRANCH_NODE_TYPES.append(ast.Match)


class CfgBuilder:
    def build_for_symbol(self, repo_path: str, file_path: str, symbol_id: str, qualname: str) -> SemanticSummary:
        target_file = Path(repo_path) / file_path
        tree = ast.parse(target_file.read_text(encoding="utf-8"), filename=str(target_file))
        function_name = qualname.split(".")[-1]
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == function_name:
                branch_points = sum(
                    1
                    for child in ast.walk(node)
                    if isinstance(child, tuple(BRANCH_NODE_TYPES))
                )
                cfg_nodes = sum(1 for _ in ast.walk(node))
                return SemanticSummary(symbol_id=symbol_id, cfg_nodes=cfg_nodes, branch_points=branch_points)
        return SemanticSummary(symbol_id=symbol_id)
