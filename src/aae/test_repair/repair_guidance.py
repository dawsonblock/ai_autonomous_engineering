from __future__ import annotations


class RepairGuidance:
    def from_failure(self, failure: dict) -> dict:
        error = str(failure.get("error", "")).lower()
        symbol = str(failure.get("symbol", "") or "")
        constraints = []
        template_families = []
        declared_intents = []
        if any(term in error for term in ["none", "null", "nonetype"]):
            constraints.append("function must safely handle None input")
            template_families.append("null_guard")
        if any(term in error for term in ["empty", "valueerror", "strip"]):
            constraints.append("empty or whitespace input must not crash")
            template_families.append("state_normalization")
        if "import" in error:
            declared_intents.append("import_change")
        if "signature" in error:
            declared_intents.append("signature_change")
        if not constraints:
            constraints.append("apply the smallest safe repair for %s" % (symbol or "target function"))
        if not template_families:
            template_families.append("bounded_edit")
        return {
            "constraints": constraints,
            "template_families": template_families,
            "declared_intents": declared_intents,
            "preferred_template": template_families[0],
        }

    def from_context(self, goal: str, recent_failures: list[str], root_symbol: str = "") -> dict:
        failure = {"error": " ".join(recent_failures) or goal, "symbol": root_symbol}
        return self.from_failure(failure)
