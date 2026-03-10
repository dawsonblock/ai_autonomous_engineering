from __future__ import annotations

from typing import Any, Dict, List


class BranchGenerator:
    def generate(self, planner_decision: dict, swarm_result: dict) -> List[Dict[str, Any]]:
        branches = []
        patch_candidates = swarm_result.get("patch_candidates", [])
        branch_by_id = {branch.get("branch_id"): branch for branch in planner_decision.get("branches", [])}
        used_branch_ids = set()
        for index, candidate in enumerate(patch_candidates, start=1):
            branch_id = (
                planner_decision.get("selected_branch_id")
                if candidate.get("plan_id") == swarm_result.get("consensus_decision", {}).get("selected_plan_id")
                else "branch_%s" % index
            )
            if not branch_id:
                branch_id = "branch_%s" % index
            if branch_id in used_branch_ids:
                branch_id = "branch_%s_%s" % (index, candidate.get("plan_id", "candidate"))
            used_branch_ids.add(branch_id)
            branch = branch_by_id.get(branch_id, {})
            selected_tests = candidate.get("simulation", {}).get("test_prediction", {}).get("affected_tests", []) or swarm_result.get("test_impact", {}).get("tests", [])
            selected_tests = _normalize_selected_tests(selected_tests)
            branches.append(
                {
                    "branch_id": branch_id,
                    "patch_candidate": candidate,
                    "selected_tests": selected_tests,
                    "metadata": branch.get("metadata", {}),
                }
            )
        return branches


def _normalize_selected_tests(paths: List[str]) -> List[str]:
    normalized = []
    seen = set()
    for path in paths:
        clean = (path or "").replace("\\", "/").strip()
        if not clean:
            continue
        if "/workspace/" in clean:
            clean = clean.split("/workspace/", 1)[1]
        clean = clean.lstrip("./")
        if clean.startswith(".sandbox_artifacts/"):
            parts = clean.split("/")
            try:
                workspace_index = parts.index("workspace")
                clean = "/".join(parts[workspace_index + 1 :])
            except ValueError:
                continue
        if clean in seen:
            continue
        seen.add(clean)
        normalized.append(clean)
    return normalized
