from __future__ import annotations

import ast
from pathlib import Path

from aae.contracts.micro_agents import PatchGenerationRequest
from aae.patching.edit_template_library import EditTemplateLibrary
from aae.patching.openai_provider import OpenAIPatchProvider


class PatchSynthesizer:
    def __init__(
        self,
        template_library: EditTemplateLibrary | None = None,
        provider: OpenAIPatchProvider | None = None,
    ) -> None:
        self.template_library = template_library or EditTemplateLibrary()
        self.provider = provider or OpenAIPatchProvider.from_env()

    def synthesize(self, repo_path: str, request: PatchGenerationRequest) -> tuple[str, str, str]:
        target_path = Path(repo_path) / request.file_path
        original_text = target_path.read_text(encoding="utf-8")
        template = self.template_library.select(request)
        template_family = template["template_family"]
        tree = ast.parse(original_text, filename=str(target_path))
        target_node = self._find_target_function(tree, request.symbol, request.target_span.start_line)
        if target_node is None:
            return original_text, original_text, template_family
        if self.provider is not None:
            try:
                body = self.provider.generate_body(
                    request=request,
                    function_signature=self._function_signature(target_node),
                    original_body=self._current_body(original_text, target_node),
                    prompt_hint=template["prompt_hint"],
                )
                updated_text = self._replace_body(original_text, target_node, body)
                return original_text, updated_text, template_family
            except Exception:
                pass
        updated_text = self._fallback_edit(original_text, request, target_node, template_family)
        return original_text, updated_text, template_family

    def _find_target_function(self, tree: ast.AST, symbol: str, start_line: int) -> ast.AST | None:
        exact = None
        first = None
        for node in ast.walk(tree):
            if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                continue
            if first is None:
                first = node
            if node.name == symbol:
                if int(getattr(node, "lineno", 0) or 0) == int(start_line or 0):
                    return node
                exact = exact or node
        return exact or first

    def _current_body(self, original_text: str, node: ast.AST) -> str:
        lines = original_text.splitlines()
        start = getattr(node.body[0], "lineno", node.lineno) - 1 if getattr(node, "body", None) else node.lineno
        end = getattr(node, "end_lineno", start)
        return "\n".join(lines[start:end])

    def _replace_body(self, original_text: str, node: ast.AST, replacement_body: str) -> str:
        lines = original_text.splitlines()
        if not getattr(node, "body", None):
            return original_text
        body_start = node.body[0].lineno - 1
        body_end = getattr(node, "end_lineno", body_start)
        indent = " " * node.body[0].col_offset
        normalized = [("%s%s" % (indent, line) if line.strip() else "") for line in replacement_body.splitlines()]
        lines[body_start:body_end] = normalized
        return "\n".join(lines) + ("\n" if original_text.endswith("\n") else "")

    def _fallback_edit(self, original_text: str, request: PatchGenerationRequest, node: ast.AST, template_family: str) -> str:
        lines = original_text.splitlines()
        indent = " " * (node.body[0].col_offset if getattr(node, "body", None) else 4)
        parameters = [arg.arg for arg in node.args.args if arg.arg != "self"]
        guard_param = parameters[0] if parameters else ""
        if template_family in {"null_guard", "state_normalization"} and guard_param:
            if "return None" in original_text:
                return_value = "None"
            elif "return {" in original_text:
                return_value = "{}"
            else:
                return_value = "None"
            guard_line = "%sif not %s:" % (indent, guard_param)
            return_line = "%s    return %s" % (indent, return_value)
            insert_at = node.body[0].lineno - 1 if node.body else getattr(node, "lineno", 0)
            if not any(guard_line.strip() == lines[index].strip() for index in range(insert_at, min(insert_at + 2, len(lines)))):
                lines[insert_at:insert_at] = [guard_line, return_line]
        elif template_family == "regression_guard":
            lines.insert(request.target_span.end_line, "%s# regression guard: %s" % (indent, request.expected_behavior))
        else:
            lines.insert(request.target_span.end_line, "%s# bounded fix: %s" % (indent, request.expected_behavior))
        return "\n".join(lines) + ("\n" if original_text.endswith("\n") else "")

    def _function_signature(self, node: ast.AST) -> str:
        args = [arg.arg for arg in node.args.args]
        return "(%s)" % ", ".join(args)
