from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent


class SymbolLocatorAgent(BaseMicroAgent):
    name = "symbol_locator"

    async def run(self, task, context):
        graph_context = context.get("graph_context", {})
        symbols = []
        for entry in graph_context.get("symbol_context", []):
            for match in entry.get("matches", []):
                symbols.append(
                    {
                        "name": match.get("name", ""),
                        "file": match.get("path", ""),
                        "line": int(match.get("line") or 0),
                    }
                )
        return {"symbols": symbols[:8]}
