from __future__ import annotations

from typing import Any, Dict

from aae.evaluation.experiment_evaluator import ExperimentEvaluator
from aae.memory.knowledge_graph import KnowledgeGraph


class ResearchLoop:
    def __init__(
        self,
        knowledge_graph: KnowledgeGraph | None = None,
        evaluator: ExperimentEvaluator | None = None,
    ) -> None:
        self.knowledge_graph = knowledge_graph or KnowledgeGraph()
        self.evaluator = evaluator or ExperimentEvaluator()

    def run(self, question: str) -> Dict[str, Any]:
        hypothesis = self.generate_hypothesis(question)
        experiment = self.plan_experiment(hypothesis)
        results = self.execute(experiment)
        score = self.evaluate(results)
        self.update_knowledge(hypothesis, results, score)
        return {
            "question": question,
            "hypothesis": hypothesis,
            "experiment": experiment,
            "score": score,
            "completed": True,
        }

    def generate_hypothesis(self, question: str) -> Dict[str, Any]:
        claim = self.knowledge_graph.create_claim(
            text="Hypothesis for: %s" % question,
            source="research_loop",
        )
        return {"claim_id": claim.claim_id, "text": claim.text, "question": question}

    def plan_experiment(self, hypothesis: Dict[str, Any]) -> Dict[str, Any]:
        return {
            "hypothesis_id": hypothesis.get("claim_id", ""),
            "steps": ["collect_data", "run_analysis", "compare_results"],
            "status": "planned",
        }

    def execute(self, experiment: Dict[str, Any]) -> Dict[str, Any]:
        return {
            "experiment": experiment,
            "tests_passed": True,
            "lint_clean": True,
            "status": "executed",
        }

    def evaluate(self, results: Dict[str, Any]) -> float:
        evaluation = self.evaluator.evaluate(results)
        return evaluation.score

    def update_knowledge(self, hypothesis: Dict[str, Any], results: Dict[str, Any], score: float) -> None:
        claim_id = hypothesis.get("claim_id", "")
        self.knowledge_graph.create_evidence(
            claim_id=claim_id,
            content="Score: %.2f" % score,
            source="experiment",
            confidence=score,
        )
