from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent


class PatchPlannerAgent(BaseMicroAgent):
    name = "patch_planner"

    async def run(self, task, context):
        candidate_file_entries = context.get("localized_files") or context.get("candidate_files", [])
        candidate_files = [item["path"] for item in candidate_file_entries]
        impacted_tests = context.get("tests", []) or context.get("impacted_tests", [])
        root_symbol = context.get("root_cause_symbol", "")
        files = candidate_files[:2] or ["unknown.py"]
        return {
            "plans": [
                {
                    "id": "plan_guard_inputs",
                    "summary": "Add a guard clause around the primary failure path%s" % (" in %s" % root_symbol if root_symbol else ""),
                    "confidence": 0.78,
                    "target_files": files,
                    "strategy": "input_validation",
                },
                {
                    "id": "plan_normalize_state",
                    "summary": "Normalize state before the risky downstream call%s" % (" in %s" % root_symbol if root_symbol else ""),
                    "confidence": 0.72,
                    "target_files": files[:1],
                    "strategy": "state_normalization",
                },
                {
                    "id": "plan_regression_test",
                    "summary": "Add or tighten regression coverage around the impacted tests",
                    "confidence": 0.62 if impacted_tests else 0.48,
                    "target_files": impacted_tests[:1] or files[:1],
                    "strategy": "test_hardening",
                },
            ]
        }
