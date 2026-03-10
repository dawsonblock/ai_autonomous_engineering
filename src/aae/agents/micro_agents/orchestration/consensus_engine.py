from __future__ import annotations

from aae.contracts.planner import CandidatePlan
from aae.contracts.ranking import RankingWeights


class ConsensusEngine:
    def __init__(self, shortlist_size: int = 3, weights: RankingWeights | None = None) -> None:
        self.shortlist_size = shortlist_size
        self.weights = weights or RankingWeights.default()

    def filter_candidates(self, candidates: list[CandidatePlan]) -> list[CandidatePlan]:
        ranked = sorted(
            candidates,
            key=lambda candidate: (
                (candidate.confidence * self.weights.confidence)
                + (candidate.predicted_test_count * self.weights.test_coverage)
                + (candidate.impact_size * self.weights.impact_penalty)
                + (candidate.risk_score * self.weights.risk_penalty)
            ),
            reverse=True,
        )
        return ranked[: self.shortlist_size]
