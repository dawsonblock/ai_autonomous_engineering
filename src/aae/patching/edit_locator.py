from __future__ import annotations

import ast
from pathlib import Path

from aae.contracts.micro_agents import PatchTargetSpan, PatchPlan


class EditLocator:
    def locate(self, repo_path: str, plan: PatchPlan, semantic_context: dict, graph_context: dict) -> list[PatchTargetSpan]:
        spans = []
        symbol_context = graph_context.get("symbol_context", [])
        preferred_symbols = []
        for entry in symbol_context:
            for match in entry.get("matches", []):
                if plan.target_files and match.get("path") not in plan.target_files:
                    continue
                preferred_symbols.append(match)
        if not preferred_symbols and plan.target_files:
            for file_path in plan.target_files:
                span = self._first_function_span(Path(repo_path) / file_path)
                if span is not None:
                    spans.append(PatchTargetSpan(file_path=file_path, symbol=span["name"], start_line=span["start"], end_line=span["end"]))
        else:
            for match in preferred_symbols[:2]:
                span = self._function_span(Path(repo_path) / match["path"], match["name"])
                if span is None:
                    continue
                spans.append(PatchTargetSpan(file_path=match["path"], symbol=match["name"], start_line=span["start"], end_line=span["end"]))
        return spans

    def _first_function_span(self, path: Path) -> dict | None:
        if not path.exists():
            return None
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                return {"name": node.name, "start": node.lineno, "end": getattr(node, "end_lineno", node.lineno)}
        return None

    def _function_span(self, path: Path, symbol: str) -> dict | None:
        if not path.exists():
            return None
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == symbol:
                return {"start": node.lineno, "end": getattr(node, "end_lineno", node.lineno)}
        return self._first_function_span(path)
