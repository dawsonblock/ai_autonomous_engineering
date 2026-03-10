from __future__ import annotations

from aae.contracts.planner import SimulationResult
from aae.behavior_model.behavior_query_engine import BehaviorQueryEngine
from aae.graph.graph_query import GraphQueryEngine
from aae.simulation.dependency_impact import DependencyImpactAnalyzer
from aae.simulation.risk_estimator import RiskEstimator
from aae.simulation.test_failure_predictor import TestFailurePredictor


class PatchSimulator:
    def __init__(
        self,
        dependency_impact: DependencyImpactAnalyzer | None = None,
        test_prediction: TestFailurePredictor | None = None,
        risk_estimator: RiskEstimator | None = None,
    ) -> None:
        self.dependency_impact = dependency_impact or DependencyImpactAnalyzer()
        self.test_prediction = test_prediction or TestFailurePredictor()
        self.risk_estimator = risk_estimator or RiskEstimator()

    def simulate(
        self,
        candidate_plan_id: str,
        changed_files: list[str],
        graph: GraphQueryEngine,
        behavior: BehaviorQueryEngine | None = None,
    ) -> SimulationResult:
        impact = self.dependency_impact.analyze(graph, changed_files, behavior=behavior)
        test_prediction = self.test_prediction.predict(graph, impact.affected_functions, changed_files, behavior=behavior)
        risk_score, risk_reasons, allow_execution = self.risk_estimator.estimate(impact, test_prediction)
        return SimulationResult(
            candidate_plan_id=candidate_plan_id,
            dependency_impact=impact,
            test_prediction=test_prediction,
            risk_score=risk_score,
            risk_reasons=risk_reasons,
            confidence=0.8 if impact.impact_size else 0.4,
            allow_execution=allow_execution,
        )
