from __future__ import annotations

from aae.contracts.planner import TestPredictionResult
from aae.graph.graph_query import GraphQueryEngine


class TestPredictionModel:
    def predict(self, graph: GraphQueryEngine, affected_functions: list[str], changed_files: list[str]) -> TestPredictionResult:
        affected_tests = []
        for function_name in affected_functions:
            symbol = function_name.split(".")[-1]
            for item in graph.tests_covering_function(symbol).items:
                affected_tests.append(item["path"])
        if not affected_tests:
            affected_tests.extend([path for path in changed_files if "test" in path])
        predicted_failures = affected_tests[:2]
        confidence = 0.75 if affected_tests else 0.35
        return TestPredictionResult(
            affected_tests=sorted(set(affected_tests)),
            predicted_failures=sorted(set(predicted_failures)),
            confidence=confidence,
        )
