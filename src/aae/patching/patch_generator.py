from __future__ import annotations

import ast
from pathlib import Path

from aae.contracts.micro_agents import PatchGenerationRequest


class HybridPatchGenerator:
    def __init__(self, llm_provider=None) -> None:
        self.llm_provider = llm_provider

    def generate(self, repo_path: str, request: PatchGenerationRequest) -> tuple[str, str]:
        target_path = Path(repo_path) / request.file_path
        original_text = target_path.read_text(encoding="utf-8")
        if self.llm_provider is not None:
            updated = self.llm_provider.generate(request=request, original_text=original_text)
            return original_text, updated
        updated = self._deterministic_edit(original_text, request)
        return original_text, updated

    def _deterministic_edit(self, original_text: str, request: PatchGenerationRequest) -> str:
        lines = original_text.splitlines()
        tree = ast.parse(original_text)
        target_node = None
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == request.symbol:
                target_node = node
                break
        if target_node is None:
            return original_text
        indent = " " * 4
        if target_node.body:
            first_stmt = target_node.body[0]
            indent = " " * first_stmt.col_offset
        params = [arg.arg for arg in target_node.args.args if arg.arg != "self"]
        guard_param = params[0] if params else ""
        return_value = self._default_return(request, original_text, target_node)
        insertion = []
        if request.strategy in {"input_validation", "state_normalization"} and guard_param:
            guard_line = "%sif not %s:" % (indent, guard_param)
            return_line = "%s    return %s" % (indent, return_value)
            block = [guard_line, return_line]
            body_start = target_node.body[0].lineno - 1
            if not any(lines[i].strip() == guard_line.strip() for i in range(target_node.lineno - 1, min(body_start + 2, len(lines)))):
                insertion = block
                lines[body_start:body_start] = insertion
        elif request.strategy == "test_hardening":
            comment = "%s# regression guard: %s" % (indent, request.expected_behavior or request.strategy)
            lines.insert(request.target_span.end_line, comment)
        else:
            comment = "%s# planned change: %s" % (indent, request.expected_behavior or request.strategy or "bounded update")
            lines.insert(request.target_span.end_line, comment)
        return "\n".join(lines) + ("\n" if original_text.endswith("\n") else "")

    def _default_return(self, request: PatchGenerationRequest, original_text: str, target_node: ast.AST) -> str:
        inferred = request.semantic_context.get("inferred_types", {})
        if any(value == "dict" for value in inferred.values()) or "return {" in original_text:
            return "{}"
        if any(value == "list" for value in inferred.values()) or "return [" in original_text:
            return "[]"
        return "None"
