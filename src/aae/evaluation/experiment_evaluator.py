from __future__ import annotations

from typing import Any, Dict, List


class EvaluationMetric:
    __slots__ = ("name", "weight")

    def __init__(self, name: str, weight: float = 1.0) -> None:
        self.name = name
        self.weight = weight


class EvaluationResult:
    __slots__ = ("score", "details", "passed")

    def __init__(self, score: float, details: Dict[str, Any], passed: bool) -> None:
        self.score = score
        self.details = details
        self.passed = passed


class ExperimentEvaluator:
    _DEFAULT_METRICS = [
        EvaluationMetric("tests_passed", weight=2.0),
        EvaluationMetric("lint_clean", weight=1.0),
        EvaluationMetric("performance", weight=1.0),
        EvaluationMetric("patch_minimal", weight=0.5),
    ]

    def __init__(self, metrics: List[EvaluationMetric] | None = None, pass_threshold: float = 0.5) -> None:
        self.metrics = metrics or list(self._DEFAULT_METRICS)
        self.pass_threshold = pass_threshold

    def evaluate(self, artifacts: Dict[str, Any]) -> EvaluationResult:
        total_score = 0.0
        max_score = 0.0
        details: Dict[str, Any] = {}

        for metric in self.metrics:
            value = artifacts.get(metric.name, False)
            score = metric.weight if value else 0.0
            total_score += score
            max_score += metric.weight
            details[metric.name] = {"value": value, "score": score, "weight": metric.weight}

        normalized = total_score / max_score if max_score > 0 else 0.0
        return EvaluationResult(
            score=normalized,
            details=details,
            passed=normalized >= self.pass_threshold,
        )

    def compare(self, result_a: EvaluationResult, result_b: EvaluationResult) -> int:
        if result_a.score > result_b.score:
            return 1
        if result_a.score < result_b.score:
            return -1
        return 0
