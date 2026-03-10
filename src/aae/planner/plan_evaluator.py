from __future__ import annotations

from aae.contracts.planner import PlanBranch


class PlanEvaluator:
    def score(self, branch: PlanBranch) -> float:
        candidate = branch.metadata.get("candidate", {})
        return (
            float(candidate.get("confidence", 0.0)) * 0.5
            + float(candidate.get("predicted_test_count", 0)) * 0.12
            - float(candidate.get("risk_score", 0.0)) * 0.35
            - float(candidate.get("impact_size", 0)) * 0.04
            - float(candidate.get("changed_line_count", 0)) * 0.01
            + (0.1 if candidate.get("simulation", {}).get("allow_execution", True) else -0.2)
        )
