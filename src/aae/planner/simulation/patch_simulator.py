from __future__ import annotations

from aae.contracts.planner import SimulationResult
from aae.graph.graph_query import GraphQueryEngine
from aae.planner.simulation.dependency_impact import DependencyImpactAnalyzer
from aae.planner.simulation.test_prediction import TestPredictionModel


class PatchSimulator:
    def __init__(
        self,
        dependency_impact: DependencyImpactAnalyzer | None = None,
        test_prediction: TestPredictionModel | None = None,
    ) -> None:
        self.dependency_impact = dependency_impact or DependencyImpactAnalyzer()
        self.test_prediction = test_prediction or TestPredictionModel()

    def simulate(self, candidate_plan_id: str, changed_files: list[str], graph: GraphQueryEngine) -> SimulationResult:
        impact = self.dependency_impact.analyze(graph, changed_files)
        test_prediction = self.test_prediction.predict(graph, impact.affected_functions, changed_files)
        risk_score = min(1.0, (impact.impact_size * 0.08) + (len(test_prediction.predicted_failures) * 0.15))
        return SimulationResult(
            candidate_plan_id=candidate_plan_id,
            dependency_impact=impact,
            test_prediction=test_prediction,
            risk_score=risk_score,
        )
