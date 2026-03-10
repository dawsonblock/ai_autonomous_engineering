from __future__ import annotations

from aae.contracts.planner import PlanBranch, PlannerAction


def build_branch(branch_id: str, candidate: dict) -> PlanBranch:
    action = PlannerAction(
        action_id="execute:%s" % candidate.get("plan_id", "unknown"),
        action_type="execute_patch_plan",
        payload=candidate,
        score=0.0,
    )
    return PlanBranch(branch_id=branch_id, actions=[action], metadata={"candidate": candidate})
