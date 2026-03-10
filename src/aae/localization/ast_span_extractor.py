from __future__ import annotations

import ast
import os
from typing import List, Optional

from .models import RankedSpan


class ASTSpanExtractor:
    def extract_spans(self, file_path: str, repo_root: str, target_function: Optional[str] = None) -> List[RankedSpan]:
        full_path = os.path.join(repo_root, file_path) if not os.path.isabs(file_path) else file_path
        if not os.path.exists(full_path):
            return []

        try:
            with open(full_path, "r", encoding="utf-8") as f:
                source = f.read()
                tree = ast.parse(source, filename=full_path)
        except Exception:
            return []

        spans: List[RankedSpan] = []

        def add_span(node: ast.AST, span_type: str, score: float = 0.5):
            start = getattr(node, "lineno", 0)
            end = getattr(node, "end_lineno", getattr(node, "lineno", 0))
            if start and end:
                spans.append(
                    RankedSpan(
                        file_path=file_path,
                        start_line=start,
                        end_line=end,
                        score=score,
                        span_type=span_type,
                        evidence={"node_type": type(node).__name__},
                        reasons=[f"AST bound: {span_type}"],
                    )
                )

        if target_function:
            target_node = None
            if "." in target_function:
                cls_name, fn_name = target_function.split(".", 1)
                for node in ast.walk(tree):
                    if isinstance(node, ast.ClassDef) and node.name == cls_name:
                        for sub in node.body:
                            if isinstance(sub, (ast.FunctionDef, ast.AsyncFunctionDef)) and sub.name == fn_name:
                                target_node = sub
                                break
            else:
                for node in ast.walk(tree):
                    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == target_function:
                        target_node = node
                        break
            
            if target_node:
                add_span(target_node, "function")
                for child in ast.walk(target_node):
                    if isinstance(child, (ast.If, ast.For, ast.While, ast.Try, ast.With)):
                        add_span(child, "control_flow")
                    elif isinstance(child, (ast.Return, ast.Raise)):
                        add_span(child, "termination")
        else:
            for node in ast.walk(tree):
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    add_span(node, "function")
                elif isinstance(node, ast.ClassDef):
                    add_span(node, "class")

        return spans
