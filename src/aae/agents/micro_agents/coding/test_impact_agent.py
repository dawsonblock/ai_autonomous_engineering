from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent


class TestImpactAgent(BaseMicroAgent):
    name = "test_impact"

    async def run(self, task, context):
        impacted = list(context.get("impacted_tests", []))
        impacted.extend(context.get("covering_tests", []))
        deduped = []
        for path in impacted:
            if path not in deduped:
                deduped.append(path)
        return {"tests": deduped[:8], "confidence": 0.8 if deduped else 0.3}
