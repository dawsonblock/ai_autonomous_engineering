from __future__ import annotations

from pydantic import BaseModel


class RankingWeights(BaseModel):
    confidence: float = 0.5
    test_coverage: float = 0.1
    impact_penalty: float = -0.05
    risk_penalty: float = -0.35

    @classmethod
    def default(cls) -> RankingWeights:
        return cls()
