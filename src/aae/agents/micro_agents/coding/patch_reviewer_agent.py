from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent


class PatchReviewerAgent(BaseMicroAgent):
    name = "patch_reviewer"

    async def run(self, task, context):
        simulation = context.get("simulation", {})
        risk_score = float(simulation.get("risk_score", 0.0))
        changed_files = context.get("changed_files", [])
        risks = []
        if risk_score > 0.65:
            risks.append("predicted risk score is elevated")
        if not changed_files:
            risks.append("patch candidate did not identify changed files")
        return {
            "accept": risk_score <= 0.65 and bool(changed_files),
            "risks": risks,
            "followups": ["add regression coverage for impacted path"] if changed_files else [],
        }
