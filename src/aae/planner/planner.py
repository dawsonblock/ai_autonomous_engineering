from __future__ import annotations

from aae.contracts.planner import PlanBranch
from aae.planner.beam_search import BeamSearch
from aae.planner.branch_memory import BranchMemory
from aae.planner.plan_evaluator import PlanEvaluator
from aae.planner.rollout_simulator import RolloutSimulator
from aae.planner.tree_search import TreeSearch


class Planner:
    def __init__(
        self,
        evaluator: PlanEvaluator | None = None,
        beam_search: BeamSearch | None = None,
        tree_search: TreeSearch | None = None,
        rollout_simulator: RolloutSimulator | None = None,
        branch_memory: BranchMemory | None = None,
    ) -> None:
        self.evaluator = evaluator or PlanEvaluator()
        self.beam_search = beam_search or BeamSearch()
        self.tree_search = tree_search or TreeSearch()
        self.rollout_simulator = rollout_simulator or RolloutSimulator()
        self.branch_memory = branch_memory or BranchMemory()

    def build_plan(self, candidates: list[dict]) -> list[PlanBranch]:
        branches = self.tree_search.expand(candidates)
        for branch in branches:
            branch.score = self.evaluator.score(branch) + self.rollout_simulator.score(branch)
            if branch.score <= 0:
                self.branch_memory.remember(branch, status="rejected", rejection_reason="non-positive rollout score")
            else:
                self.branch_memory.remember(branch, status="explored")
        shortlisted = self.beam_search.prune([branch for branch in branches if branch.score > 0])
        for branch in shortlisted:
            branch.metadata["search_score"] = branch.score
        return shortlisted
