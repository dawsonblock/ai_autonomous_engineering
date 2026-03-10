from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent
from aae.test_repair.repair_guidance import RepairGuidance


class PatchPlannerAgent(BaseMicroAgent):
    name = "patch_planner"

    def __init__(self, repair_guidance: RepairGuidance | None = None) -> None:
        self.repair_guidance = repair_guidance or RepairGuidance()

    async def run(self, task, context):
        candidate_file_entries = context.get("localized_files") or context.get("candidate_files", [])
        candidate_files = [item["path"] for item in candidate_file_entries]
        impacted_tests = context.get("tests", []) or context.get("impacted_tests", [])
        root_symbol = context.get("root_cause_symbol", "")
        repair_guidance = context.get("repair_guidance") or self.repair_guidance.from_context(
            goal=str(task.get("goal", "")),
            recent_failures=list(context.get("recent_failures", [])),
            root_symbol=root_symbol,
        )
        preferred_template = repair_guidance.get("preferred_template", "bounded_edit")
        declared_intents = list(repair_guidance.get("declared_intents", []))
        files = candidate_files[:2] or ["unknown.py"]
        return {
            "plans": [
                {
                    "id": "plan_guard_inputs",
                    "summary": "Add a guard clause around the primary failure path%s" % (" in %s" % root_symbol if root_symbol else ""),
                    "confidence": 0.84 if preferred_template == "null_guard" else 0.78,
                    "target_files": files,
                    "strategy": "input_validation",
                    "template_family": "null_guard",
                    "declared_intents": declared_intents,
                    "repair_guidance": repair_guidance,
                },
                {
                    "id": "plan_normalize_state",
                    "summary": "Normalize state before the risky downstream call%s" % (" in %s" % root_symbol if root_symbol else ""),
                    "confidence": 0.8 if preferred_template == "state_normalization" else 0.72,
                    "target_files": files[:1],
                    "strategy": "state_normalization",
                    "template_family": "state_normalization",
                    "declared_intents": declared_intents,
                    "repair_guidance": repair_guidance,
                },
                {
                    "id": "plan_regression_test",
                    "summary": "Add or tighten regression coverage around the impacted tests",
                    "confidence": 0.62 if impacted_tests else 0.48,
                    "target_files": impacted_tests[:1] or files[:1],
                    "strategy": "test_hardening",
                    "template_family": "regression_guard",
                    "declared_intents": declared_intents,
                    "repair_guidance": repair_guidance,
                },
            ]
        }
