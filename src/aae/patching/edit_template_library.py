from __future__ import annotations

from typing import Dict

from aae.contracts.micro_agents import PatchGenerationRequest


class EditTemplateLibrary:
    def select(self, request: PatchGenerationRequest) -> Dict[str, str]:
        templates = {
            "input_validation": {
                "template_family": "null_guard",
                "prompt_hint": "Add a minimal guard for missing or empty inputs before the risky operation.",
            },
            "state_normalization": {
                "template_family": "state_normalization",
                "prompt_hint": "Normalize input or intermediate state before the downstream call.",
            },
            "error_propagation": {
                "template_family": "error_propagation",
                "prompt_hint": "Preserve errors explicitly and avoid unsafe fall-through behavior.",
            },
            "test_hardening": {
                "template_family": "regression_guard",
                "prompt_hint": "Add a minimal regression assertion or guard comment without broad rewrites.",
            },
        }
        selected = templates.get(request.strategy) or {
            "template_family": request.template_family or "bounded_edit",
            "prompt_hint": "Make the smallest possible bounded fix for the requested behavior.",
        }
        return {
            "template_family": request.template_family or selected["template_family"],
            "prompt_hint": selected["prompt_hint"],
        }
