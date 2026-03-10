from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent


class PatchPlannerAgent(BaseMicroAgent):
    name = "patch_planner"

    async def run(self, task, context):
        candidate_files = [item["path"] for item in context.get("candidate_files", [])]
        impacted_tests = context.get("impacted_tests", [])
        files = candidate_files[:2] or ["unknown.py"]
        return {
            "plans": [
                {
                    "id": "plan_guard_inputs",
                    "summary": "Add a guard clause around the primary failure path",
                    "confidence": 0.76,
                    "target_files": files,
                    "strategy": "input_validation",
                },
                {
                    "id": "plan_normalize_state",
                    "summary": "Normalize state before the risky downstream call",
                    "confidence": 0.7,
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
