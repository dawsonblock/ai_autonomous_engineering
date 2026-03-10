from __future__ import annotations

import ast
from pathlib import Path

from aae.contracts.micro_agents import PatchTargetSpan, PatchPlan


class EditLocator:
    def locate(self, repo_path: str, plan: PatchPlan, semantic_context: dict, graph_context: dict) -> list[PatchTargetSpan]:
        spans = []
        suspicious_locations = graph_context.get("suspicious_locations", [])
        for location in suspicious_locations:
            normalized_path = self._normalize_file_path(repo_path, location.get("file_path", ""))
            if not normalized_path:
                continue
            if plan.target_files and normalized_path not in plan.target_files:
                continue
            path = Path(repo_path) / normalized_path
            resolved = self._function_span(path, location.get("symbol", "")) if location.get("symbol") else None
            start_line = int((resolved or {}).get("start") or location.get("start_line") or 0)
            end_line = int((resolved or {}).get("end") or location.get("end_line") or start_line)
            if start_line and end_line:
                spans.append(
                    PatchTargetSpan(
                        file_path=normalized_path,
                        symbol=location.get("symbol", ""),
                        start_line=start_line,
                        end_line=end_line,
                    )
                )
        if spans:
            return spans[:2]
        symbol_context = graph_context.get("symbol_context", [])
        preferred_symbols = []
        for entry in symbol_context:
            for match in entry.get("matches", []):
                normalized_path = self._normalize_file_path(repo_path, match.get("path", ""))
                if not normalized_path:
                    continue
                if plan.target_files and normalized_path not in plan.target_files:
                    continue
                preferred_symbols.append({**match, "path": normalized_path})
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

    def _normalize_file_path(self, repo_path: str, value: str) -> str:
        raw = (value or "").strip()
        if not raw:
            return ""
        repo_root = Path(repo_path).resolve()
        path = Path(raw)
        if path.is_absolute():
            try:
                return path.resolve().relative_to(repo_root).as_posix()
            except ValueError:
                return ""
        marker = "/workspace/"
        normalized = raw.replace("\\", "/")
        if marker in normalized:
            normalized = normalized.split(marker, 1)[1]
        normalized = normalized.lstrip("./")
        if normalized.startswith(".sandbox_artifacts/"):
            parts = normalized.split("/")
            try:
                workspace_index = parts.index("workspace")
                normalized = "/".join(parts[workspace_index + 1 :])
            except ValueError:
                return ""
        return normalized

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
