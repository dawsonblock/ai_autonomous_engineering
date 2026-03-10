from __future__ import annotations

import ast
import json
import os
from typing import Any, Dict, List, Optional

from .models import CoverageRecord


class CoverageLoader:
    def __init__(self):
        self._ast_cache: Dict[str, Any] = {}

    def _get_function_at_line(self, file_path: str, line_number: int, repo_root: str) -> Optional[str]:
        full_path = os.path.join(repo_root, file_path) if not os.path.isabs(file_path) else file_path
        if not os.path.exists(full_path):
            return None

        if full_path not in self._ast_cache:
            try:
                with open(full_path, "r", encoding="utf-8") as f:
                    self._ast_cache[full_path] = ast.parse(f.read(), filename=full_path)
            except Exception:
                self._ast_cache[full_path] = None
        
        tree = self._ast_cache[full_path]
        if not tree:
            return None

        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                if getattr(node, "lineno", -1) <= line_number <= getattr(node, "end_lineno", float('inf')):
                    return node.name
        return None

    def load(self, context: Dict[str, Any]) -> List[CoverageRecord]:
        records: List[CoverageRecord] = []
        coverage_data = context.get("coverage", [])
        repo_root = context.get("repo_root", "")

        if isinstance(coverage_data, str) and os.path.exists(coverage_data):
            try:
                with open(coverage_data, "r") as f:
                    coverage_data = json.load(f)
            except Exception:
                coverage_data = []

        if isinstance(coverage_data, list):
            for item in coverage_data:
                file_path = item.get("file_path", "")
                test_name = item.get("test_name", "unknown")
                line_hits = item.get("line_hits", [])
                
                # Aggregate line hits into functions
                fn_hits: Dict[str, List[int]] = {}
                for line in line_hits:
                    fn_name = self._get_function_at_line(file_path, line, repo_root) or "global"
                    if fn_name not in fn_hits:
                        fn_hits[fn_name] = []
                    fn_hits[fn_name].append(line)
                    
                for fn, hits in fn_hits.items():
                    records.append(
                        CoverageRecord(
                            test_name=test_name,
                            file_path=file_path,
                            function_name=fn if fn != "global" else None,
                            line_hits=hits,
                        )
                    )
        return records
