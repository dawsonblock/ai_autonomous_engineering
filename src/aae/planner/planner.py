from __future__ import annotations

from aae.contracts.planner import PlanBranch
from aae.planner.action_tree import build_branch
from aae.planner.beam_search import BeamSearch
from aae.planner.plan_evaluator import PlanEvaluator


class Planner:
    def __init__(
        self,
        evaluator: PlanEvaluator | None = None,
        beam_search: BeamSearch | None = None,
    ) -> None:
        self.evaluator = evaluator or PlanEvaluator()
        self.beam_search = beam_search or BeamSearch()

    def build_plan(self, candidates: list[dict]) -> list[PlanBranch]:
        branches = []
        for index, candidate in enumerate(candidates, start=1):
            branch = build_branch("branch_%s" % index, candidate)
            branch.score = self.evaluator.score(branch)
            branches.append(branch)
        return self.beam_search.prune(branches)
