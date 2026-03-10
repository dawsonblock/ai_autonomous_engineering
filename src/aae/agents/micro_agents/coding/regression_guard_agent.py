from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent


class RegressionGuardAgent(BaseMicroAgent):
    name = "regression_guard"

    async def run(self, task, context):
        syntax_valid = bool(context.get("syntax_valid"))
        constraint_results = context.get("constraint_results", [])
        risk_score = float(context.get("risk_score", 0.0))
        reasons = [result.get("details", "") for result in constraint_results if not result.get("passed")]
        if risk_score > 0.7:
            reasons.append("simulation risk score too high")
        return {
            "accept": syntax_valid and not reasons,
            "boundary_ok": not reasons,
            "reasons": [reason for reason in reasons if reason],
        }
