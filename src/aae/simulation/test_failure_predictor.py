from __future__ import annotations

from aae.behavior_model.behavior_query_engine import BehaviorQueryEngine
from aae.contracts.planner import TestPredictionResult
from aae.graph.graph_query import GraphQueryEngine


class TestFailurePredictor:
    def predict(
        self,
        graph: GraphQueryEngine,
        affected_functions: list[str],
        changed_files: list[str],
        behavior: BehaviorQueryEngine | None = None,
    ) -> TestPredictionResult:
        affected_tests = []
        for function_name in affected_functions:
            symbol = function_name.split(".")[-1]
            for item in graph.tests_covering_function(symbol).items:
                affected_tests.append(item["path"])
        if behavior is not None:
            for item in behavior.trace_overlap(affected_functions).items:
                if item["file_path"] and "test" in item["file_path"]:
                    affected_tests.append(item["file_path"])
        if not affected_tests:
            affected_tests.extend([path for path in changed_files if "test" in path])
        predicted_failures = affected_tests[: max(1, min(3, len(affected_tests)))]
        confidence = 0.8 if affected_tests else 0.35
        return TestPredictionResult(
            affected_tests=sorted(set(affected_tests)),
            predicted_failures=sorted(set(predicted_failures)),
            confidence=confidence,
        )
