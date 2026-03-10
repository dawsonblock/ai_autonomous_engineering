from __future__ import annotations


class CounterexampleGenerator:
    def generate(self, failure: dict, selected_tests: list[str] | None = None) -> list[dict]:
        selected_tests = selected_tests or []
        symbol = str(failure.get("symbol", "value") or "value")
        error = str(failure.get("error", "")).lower()
        candidates = []
        if any(term in error for term in ["none", "null", "nonetype"]):
            candidates.append({"name": "none_input", "input": None, "reason": "none handling"})
        if any(term in error for term in ["empty", "valueerror", "whitespace", "strip"]):
            candidates.extend(
                [
                    {"name": "empty_string", "input": "", "reason": "empty input"},
                    {"name": "whitespace_only", "input": "   ", "reason": "whitespace input"},
                ]
            )
        if not candidates:
            candidates.append({"name": "default_regression", "input": "", "reason": "generic guard regression"})
        for candidate in candidates:
            candidate["symbol"] = symbol
            candidate["selected_tests"] = list(selected_tests)
        return candidates[:4]
