from __future__ import annotations

from aae.contracts.planner import DependencyImpactResult, TestPredictionResult


class RiskEstimator:
    def estimate(self, impact: DependencyImpactResult, prediction: TestPredictionResult) -> tuple[float, list[str], bool]:
        risk_score = min(1.0, (impact.impact_size * 0.06) + (len(prediction.predicted_failures) * 0.18))
        reasons = []
        if impact.impact_size > 3:
            reasons.append("broad dependency impact")
        if prediction.predicted_failures:
            reasons.append("predicted failing tests: %s" % ", ".join(prediction.predicted_failures))
        allow_execution = risk_score < 0.75
        return risk_score, reasons, allow_execution
