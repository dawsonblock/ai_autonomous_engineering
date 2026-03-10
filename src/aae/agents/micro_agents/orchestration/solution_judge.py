from __future__ import annotations

from aae.contracts.planner import CandidatePlan, ConsensusDecision, JudgeScoreBreakdown


class SolutionJudge:
    def select_best(self, candidates: list[CandidatePlan]) -> ConsensusDecision:
        best = None
        breakdowns = []
        for candidate in candidates:
            components = {
                "confidence": candidate.confidence * 0.5,
                "test_coverage": candidate.predicted_test_count * 0.1,
                "impact_penalty": candidate.impact_size * -0.05,
                "risk_penalty": candidate.risk_score * -0.35,
            }
            total = sum(components.values())
            breakdowns.append(
                JudgeScoreBreakdown(
                    plan_id=candidate.plan_id,
                    total_score=total,
                    components=components,
                )
            )
            if best is None or total > best[1]:
                best = (candidate, total)
        return ConsensusDecision(
            selected_plan_id=best[0].plan_id if best is not None else "",
            shortlisted_plan_ids=[candidate.plan_id for candidate in candidates],
            score_breakdowns=breakdowns,
        )
