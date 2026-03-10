from __future__ import annotations

from aae.contracts.planner import PlannerDecision, PlannerState
from aae.planner.planner import Planner


class PlannerRuntime:
    def __init__(self, planner: Planner | None = None) -> None:
        self.planner = planner or Planner()

    def plan(self, workflow_goal: str, graph_context: dict, memory_state: dict, swarm_result: dict) -> PlannerDecision:
        planner_state = PlannerState(
            workflow_goal=workflow_goal,
            graph_context=graph_context,
            memory_state=memory_state,
            prior_failures=[
                value.get("error", {}).get("message", "")
                for value in (memory_state.get("task_results") or {}).values()
                if value.get("error")
            ],
            tool_hints=swarm_result.get("tool_recommendations", {}),
        )
        candidates = []
        for candidate in swarm_result.get("shortlisted_candidates", []):
            candidate = dict(candidate)
            simulation = swarm_result.get("simulation", {})
            candidate.setdefault("predicted_test_count", len(simulation.get("test_prediction", {}).get("affected_tests", [])))
            candidate.setdefault("risk_score", float(simulation.get("risk_score", 0.0)))
            candidates.append(candidate)
        if not candidates and swarm_result.get("selected_plan"):
            selected = dict(swarm_result["selected_plan"])
            selected["plan_id"] = selected.get("id", "")
            candidates.append(selected)
        branches = self.planner.build_plan(candidates)
        selected_branch_id = branches[0].branch_id if branches else ""
        return PlannerDecision(
            selected_branch_id=selected_branch_id,
            branches=branches,
            rationale={"goal": workflow_goal, "candidate_count": len(candidates), "state": planner_state.model_dump(mode="json")},
        )
