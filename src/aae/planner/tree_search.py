from __future__ import annotations

from aae.contracts.planner import PlanBranch, PlannerAction


class TreeSearch:
    def expand(self, candidates: list[dict]) -> list[PlanBranch]:
        branches = []
        for index, candidate in enumerate(candidates, start=1):
            actions = [
                PlannerAction(
                    action_id="query_behavior:%s" % index,
                    action_type="query_behavior_model",
                    payload={"plan_id": candidate.get("plan_id", ""), "suspicious_locations": candidate.get("suspicious_locations", [])},
                ),
                PlannerAction(
                    action_id="localize_failure:%s" % index,
                    action_type="localize_failure",
                    payload={"plan_id": candidate.get("plan_id", ""), "evidence": candidate.get("evidence", [])},
                ),
                PlannerAction(
                    action_id="choose_strategy:%s" % index,
                    action_type="choose_strategy",
                    payload={"plan_id": candidate.get("plan_id", ""), "template_family": candidate.get("template_family", "")},
                ),
                PlannerAction(
                    action_id="reference_lookup:%s" % index,
                    action_type="reference_lookup",
                    payload={"plan_id": candidate.get("plan_id", ""), "related_symbols": candidate.get("related_symbols", [])},
                ),
                PlannerAction(
                    action_id="rank_context:%s" % index,
                    action_type="rank_context_selection",
                    payload={"plan_id": candidate.get("plan_id", ""), "ranked_files": candidate.get("ranked_files", [])},
                ),
                PlannerAction(
                    action_id="repair_guidance:%s" % index,
                    action_type="repair_guidance",
                    payload={"plan_id": candidate.get("plan_id", ""), "repair_guidance": candidate.get("repair_guidance", {})},
                ),
                PlannerAction(
                    action_id="generate_patch:%s" % index,
                    action_type="generate_patch",
                    payload=candidate,
                ),
                PlannerAction(
                    action_id="git_apply:%s" % index,
                    action_type="git_patch_apply",
                    payload={"plan_id": candidate.get("plan_id", ""), "declared_intents": candidate.get("declared_intents", [])},
                ),
                PlannerAction(
                    action_id="sandbox_check:%s" % index,
                    action_type="run_sandbox_check",
                    payload={"plan_id": candidate.get("plan_id", ""), "selected_tests": candidate.get("selected_tests", [])},
                ),
            ]
            if float(candidate.get("risk_score", 0.0)) > 0.45:
                actions.append(
                    PlannerAction(
                        action_id="refine_patch:%s" % index,
                        action_type="refine_patch",
                        payload={"plan_id": candidate.get("plan_id", ""), "repair_guidance": candidate.get("repair_guidance", {})},
                    )
                )
                actions.append(
                    PlannerAction(
                        action_id="counterexample_refinement:%s" % index,
                        action_type="counterexample_refinement",
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
                        "suspicion_inputs": candidate.get("suspicious_locations", []),
                        "patch_metadata": {
                            "changed_files": candidate.get("changed_files", []),
                            "changed_line_count": candidate.get("changed_line_count", 0),
                            "template_family": candidate.get("template_family", ""),
                        },
                        "repair_guidance": candidate.get("repair_guidance", {}),
                        "simulation_summary": candidate.get("simulation", {}),
                    },
                )
            )
        return branches
