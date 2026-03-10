from __future__ import annotations

from aae.contracts.planner import CandidatePlan, ConsensusDecision, JudgeScoreBreakdown
from aae.contracts.ranking import RankingWeights
from aae.agents.micro_agents.orchestration.openai_judge import OpenAIJudgeProvider


class SolutionJudge:
    def __init__(self, provider: OpenAIJudgeProvider | None = None, weights: RankingWeights | None = None) -> None:
        self.provider = provider or OpenAIJudgeProvider.from_env()
        self.weights = weights or RankingWeights.default()

    def select_best(self, candidates: list[CandidatePlan]) -> ConsensusDecision:
        if not candidates:
            return ConsensusDecision()
            
        best_id = ""
        breakdowns = []
        best_scalar = None
        
        # 1. Evaluate all candidates algebraically to provide breakdowns
        for candidate in candidates:
            components = {
                "confidence": candidate.confidence * self.weights.confidence,
                "test_coverage": candidate.predicted_test_count * self.weights.test_coverage,
                "impact_penalty": candidate.impact_size * self.weights.impact_penalty,
                "risk_penalty": candidate.risk_score * self.weights.risk_penalty,
            }
            total = sum(components.values())
            breakdowns.append(
                JudgeScoreBreakdown(
                    plan_id=candidate.plan_id,
                    total_score=total,
                    components=components,
                )
            )
            if best_scalar is None or total > best_scalar[1]:
                best_scalar = (candidate, total)
                
        # 2. Let the LLM judge the candidate plans if available
        if self.provider is not None and len(candidates) > 1:
            try:
                best_id = self.provider.select_best_plan(candidates)
            except Exception:
                best_id = best_scalar[0].plan_id if best_scalar else candidates[0].plan_id
        else:
            best_id = best_scalar[0].plan_id if best_scalar else candidates[0].plan_id

        return ConsensusDecision(
            selected_plan_id=best_id,
            shortlisted_plan_ids=[candidate.plan_id for candidate in candidates],
            score_breakdowns=breakdowns,
        )
