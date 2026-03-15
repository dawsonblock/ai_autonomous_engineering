from __future__ import annotations

from typing import Any, Dict

from aae.agents.micro_agents.base import BaseMicroAgent
from aae.evaluation.experiment_evaluator import ExperimentEvaluator


class CriticAgent(BaseMicroAgent):
    name = "critic"
    domain = "evaluation"

    def __init__(self, evaluator: ExperimentEvaluator | None = None) -> None:
        self.evaluator = evaluator or ExperimentEvaluator()

    async def run(self, task: Dict[str, Any], context: Dict[str, Any]) -> Dict[str, Any]:
        artifacts = task.get("artifacts", {})
        result = self.evaluator.evaluate(artifacts)
        return {
            "score": result.score,
            "passed": result.passed,
            "details": result.details,
        }
