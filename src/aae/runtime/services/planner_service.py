from __future__ import annotations

import json
from pathlib import Path

from aae.exploration.branch_generator import BranchGenerator
from aae.exploration.experiment_runner import ExperimentRunner
from aae.exploration.result_comparator import ResultComparator
from aae.planner.planner_runtime import PlannerRuntime


class PlannerService:
    def __init__(
        self,
        artifacts_dir: str,
        planner_runtime: PlannerRuntime | None = None,
        branch_generator: BranchGenerator | None = None,
        experiment_runner: ExperimentRunner | None = None,
        result_comparator: ResultComparator | None = None,
    ) -> None:
        self.artifacts_dir = artifacts_dir
        self.planner_runtime = planner_runtime or PlannerRuntime()
        self.branch_generator = branch_generator or BranchGenerator()
        self.experiment_runner = experiment_runner or ExperimentRunner()
        self.result_comparator = result_comparator or ResultComparator()

    async def decide_and_explore(self, workflow_id: str, goal: str, repo_path: str, graph_context: dict, memory_state: dict, swarm_result: dict):
        meta_profile = self.load_meta_profile()
        planner_decision = self.planner_runtime.plan(
            workflow_goal=goal,
            graph_context=graph_context,
            memory_state={**memory_state, "meta_strategy_profile": meta_profile},
            swarm_result=swarm_result,
        )
        planner_decision.rationale["meta_strategy_profile"] = meta_profile
        exploration_branches = self.branch_generator.generate(planner_decision.model_dump(mode="json"), swarm_result)
        exploration_results = await self.experiment_runner.run(
            repo_path=repo_path,
            branches=exploration_branches,
            artifacts_dir=str(Path(self.artifacts_dir) / "sandbox" / workflow_id),
        )
        branch_comparison = self.result_comparator.compare(exploration_results).model_dump(mode="json")
        branch_memory_records = [record.model_dump(mode="json") for record in self.planner_runtime.planner.branch_memory.records()]
        execution_by_branch = {result.get("branch_id", ""): result.get("execution", {}).get("metadata", {}) for result in exploration_results}
        for record in branch_memory_records:
            execution_metadata = execution_by_branch.get(record.get("branch_id", ""), {})
            if execution_metadata:
                record["metadata"] = {
                    **record.get("metadata", {}),
                    "patch_apply_status": execution_metadata.get("patch_apply_status", ""),
                    "rollback_status": execution_metadata.get("rollback_status", ""),
                    "repair_loop": execution_metadata.get("repair_loop", {}),
                    "counterexample_paths": execution_metadata.get("counterexample_paths", []),
                    "execution_mode": execution_metadata.get("execution_mode", ""),
                    "trust_level": execution_metadata.get("trust_level", ""),
                }
        return planner_decision, exploration_results, branch_comparison, branch_memory_records

    def load_meta_profile(self) -> dict:
        profile_path = Path(self.artifacts_dir) / "meta" / "strategy_profile.json"
        if not profile_path.exists():
            return {}
        try:
            return json.loads(profile_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            return {}
