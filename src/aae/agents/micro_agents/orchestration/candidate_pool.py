from __future__ import annotations

from aae.contracts.planner import CandidatePlan


class CandidatePool:
    def __init__(self) -> None:
        self.candidates: list[CandidatePlan] = []

    def add(self, candidate: CandidatePlan) -> None:
        self.candidates.append(candidate)

    def get_all(self) -> list[CandidatePlan]:
        return list(self.candidates)

    def clear(self) -> None:
        self.candidates.clear()
