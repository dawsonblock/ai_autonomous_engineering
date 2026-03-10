from __future__ import annotations

from aae.contracts.planner import CandidatePlan


class ConsensusEngine:
    def __init__(self, shortlist_size: int = 3) -> None:
        self.shortlist_size = shortlist_size

    def filter_candidates(self, candidates: list[CandidatePlan]) -> list[CandidatePlan]:
        ranked = sorted(
            candidates,
            key=lambda candidate: (
                candidate.confidence
                + (candidate.predicted_test_count * 0.05)
                + (candidate.impact_size * -0.03)
                + (candidate.risk_score * -0.4)
            ),
            reverse=True,
        )
        return ranked[: self.shortlist_size]
