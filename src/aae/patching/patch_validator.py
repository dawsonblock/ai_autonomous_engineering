from __future__ import annotations

import ast
from pathlib import Path

from aae.contracts.micro_agents import PatchConstraintResult, PatchTargetSpan, PatchValidationResult


class PatchValidator:
    def validate(
        self,
        file_path: str,
        original_text: str,
        updated_text: str,
        target_spans: list[PatchTargetSpan],
        max_files: int = 1,
        max_changed_lines: int = 20,
        declared_intents: list[str] | None = None,
    ) -> PatchValidationResult:
        errors = []
        constraint_results = []
        syntax_valid = True
        declared_intents = declared_intents or []
        try:
            updated_tree = ast.parse(updated_text)
            original_tree = ast.parse(original_text)
        except SyntaxError as exc:
            syntax_valid = False
            errors.append(str(exc))
            updated_tree = None
            original_tree = None

        if len({file_path}) > max_files:
            constraint_results.append(PatchConstraintResult(name="file_budget", passed=False, details="changed file budget exceeded"))
        else:
            constraint_results.append(PatchConstraintResult(name="file_budget", passed=True))

        if syntax_valid and original_tree is not None and updated_tree is not None:
            original_sigs = _function_signatures(original_tree)
            updated_sigs = _function_signatures(updated_tree)
            if original_sigs != updated_sigs:
                if "signature_change" not in declared_intents:
                    errors.append("function signatures changed outside declared intent")
                    constraint_results.append(PatchConstraintResult(name="signature_stability", passed=False, details="signature changed"))
                else:
                    constraint_results.append(PatchConstraintResult(name="signature_stability", passed=True, details="declared signature change"))
            else:
                constraint_results.append(PatchConstraintResult(name="signature_stability", passed=True))

            if _import_statements(original_tree) != _import_statements(updated_tree):
                if "import_change" not in declared_intents:
                    errors.append("imports changed outside declared intent")
                    constraint_results.append(PatchConstraintResult(name="import_stability", passed=False, details="imports changed"))
                else:
                    constraint_results.append(PatchConstraintResult(name="import_stability", passed=True, details="declared import change"))
            else:
                constraint_results.append(PatchConstraintResult(name="import_stability", passed=True))

        changed_lines = _changed_line_numbers(original_text, updated_text)
        if len(changed_lines) > max_changed_lines:
            errors.append("patch exceeded changed-line budget")
            constraint_results.append(
                PatchConstraintResult(
                    name="line_budget",
                    passed=False,
                    details="changed %s lines" % len(changed_lines),
                )
            )
        else:
            constraint_results.append(PatchConstraintResult(name="line_budget", passed=True, details="changed %s lines" % len(changed_lines)))
        allowed = set()
        for span in target_spans:
            allowed.update(range(max(1, span.start_line - 1), span.end_line + 3))
        out_of_scope = [line for line in changed_lines if line not in allowed]
        if out_of_scope:
            errors.append("edits escaped approved span")
            constraint_results.append(PatchConstraintResult(name="span_boundary", passed=False, details="changed lines: %s" % out_of_scope))
        else:
            constraint_results.append(PatchConstraintResult(name="span_boundary", passed=True))

        return PatchValidationResult(
            syntax_valid=syntax_valid,
            passed=syntax_valid and not errors,
            errors=errors,
            constraint_results=constraint_results,
        )


def _function_signatures(tree: ast.AST) -> dict[str, list[str]]:
    signatures = {}
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            signatures[node.name] = [arg.arg for arg in node.args.args]
    return signatures


def _import_statements(tree: ast.AST) -> list[str]:
    imports = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            imports.extend(alias.name for alias in node.names)
        elif isinstance(node, ast.ImportFrom):
            imports.append("%s:%s" % (node.module, ",".join(alias.name for alias in node.names)))
    return sorted(imports)


def _changed_line_numbers(original_text: str, updated_text: str) -> list[int]:
    original_lines = original_text.splitlines()
    updated_lines = updated_text.splitlines()
    changed = []
    for index, (left, right) in enumerate(zip(original_lines, updated_lines), start=1):
        if left != right:
            changed.append(index)
    if len(updated_lines) > len(original_lines):
        changed.extend(range(len(original_lines) + 1, len(updated_lines) + 1))
    return changed
