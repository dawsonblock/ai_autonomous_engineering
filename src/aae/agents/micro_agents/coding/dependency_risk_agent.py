from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent


class DependencyRiskAgent(BaseMicroAgent):
    name = "dependency_risk"

    async def run(self, task, context):
        graph = context["graph"]
        risk_score = 0.0
        reasons = []
        for symbol in context.get("symbols", [])[:3]:
            matches = graph.find_functions(symbol.get("name", "")).items
            for match in matches:
                outgoing = len(graph.outgoing.get(match["id"], []))
                incoming = len(graph.incoming.get(match["id"], []))
                if outgoing + incoming > 4:
                    risk_score += 0.2
                    reasons.append("%s has wide graph connectivity" % match["qualname"])
        if context.get("imported_modules"):
            risk_score += 0.1
            reasons.append("import chain may widen patch impact")
        return {"risk_score": min(1.0, risk_score), "risk_reasons": reasons}
