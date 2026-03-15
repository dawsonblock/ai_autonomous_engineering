from __future__ import annotations

from typing import Any, Dict

from aae.agents.micro_agents.base import BaseMicroAgent
from aae.planner.long_horizon_planner import LongHorizonPlanner


class PlannerAgent(BaseMicroAgent):
    name = "planner"
    domain = "planning"

    def __init__(self, planner: LongHorizonPlanner | None = None) -> None:
        self.planner = planner or LongHorizonPlanner()

    async def run(self, task: Dict[str, Any], context: Dict[str, Any]) -> Dict[str, Any]:
        goal = task.get("goal", "")
        steps = self.planner.plan(goal, context)
        return {
            "goal": goal,
            "steps": [
                {"step_id": step.step_id, "action": step.action, "depends_on": step.depends_on}
                for step in steps
            ],
            "step_count": len(steps),
        }
