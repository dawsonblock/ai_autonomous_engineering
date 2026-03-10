from __future__ import annotations

from aae.contracts.planner import PlanBranch


class BeamSearch:
    def prune(self, branches: list[PlanBranch], width: int = 3) -> list[PlanBranch]:
        return sorted(branches, key=lambda branch: branch.score, reverse=True)[:width]
