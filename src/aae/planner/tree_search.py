from __future__ import annotations

from aae.contracts.planner import PlanBranch, PlannerAction


class TreeSearch:
    def expand(self, candidates: list[dict]) -> list[PlanBranch]:
        branches = []
        for index, candidate in enumerate(candidates, start=1):
            actions = [
                PlannerAction(
                    action_id="query_graph:%s" % index,
                    action_type="query_graph",
                    payload={"plan_id": candidate.get("plan_id", "")},
                ),
                PlannerAction(
                    action_id="generate_patch:%s" % index,
                    action_type="generate_patch",
                    payload=candidate,
                ),
            ]
            if float(candidate.get("risk_score", 0.0)) > 0.45:
                actions.append(
                    PlannerAction(
                        action_id="refine_patch:%s" % index,
                        action_type="refine_patch",
                        payload={"plan_id": candidate.get("plan_id", "")},
                    )
                )
            branches.append(
                PlanBranch(
                    branch_id="branch_%s" % index,
                    actions=actions,
                    metadata={
                        "candidate": candidate,
                        "action_sequence": [action.action_type for action in actions],
                        "branch_depth": len(actions),
                    },
                )
            )
        return branches
