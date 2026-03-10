from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent


class DependencyTracerAgent(BaseMicroAgent):
    name = "dependency_tracer"

    async def run(self, task, context):
        graph_context = context.get("graph_context", {})
        symbols = context.get("symbols", [])
        return {
            "call_chain": graph_context.get("call_chains", [])[:6],
            "impacted_tests": graph_context.get("covering_tests", [])[:6],
            "imported_modules": [symbol.get("name", "") for symbol in symbols[:3]],
        }
