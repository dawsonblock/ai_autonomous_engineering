from __future__ import annotations

from aae.agents.micro_agents.orchestration.swarm_controller import SwarmController
from aae.learning.tool_router import ToolRouter


class SwarmService:
    def __init__(
        self,
        swarm: SwarmController | None = None,
        tool_router: ToolRouter | None = None,
    ) -> None:
        self.swarm = swarm or SwarmController()
        self.tool_router = tool_router or ToolRouter()

    def route_tools(self, task_type: str, goal: str, graph_context: dict, memory_state: dict) -> dict:
        return self.tool_router.route(
            task_state={"task_type": task_type, "goal": goal},
            graph_context=graph_context,
            prior_actions=memory_state.get("planner_actions", []),
            recent_failures=[
                value.get("error", {}).get("message", "")
                for value in (memory_state.get("task_results") or {}).values()
                if value.get("error")
            ],
        )

    async def run(self, task: dict, context: dict) -> dict:
        return await self.swarm.run(task=task, context=context)
