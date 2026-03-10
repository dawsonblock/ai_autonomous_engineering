from __future__ import annotations

from aae.test_repair.counterexample_generator import CounterexampleGenerator
from aae.test_repair.failure_analyzer import FailureAnalyzer
from aae.test_repair.repair_guidance import RepairGuidance
from aae.test_repair.test_mutator import TestMutator


class RepairLoop:
    def __init__(
        self,
        failure_analyzer: FailureAnalyzer | None = None,
        counterexample_generator: CounterexampleGenerator | None = None,
        test_mutator: TestMutator | None = None,
        repair_guidance: RepairGuidance | None = None,
        max_iterations: int = 3,
    ) -> None:
        self.failure_analyzer = failure_analyzer or FailureAnalyzer()
        self.counterexample_generator = counterexample_generator or CounterexampleGenerator()
        self.test_mutator = test_mutator or TestMutator()
        self.repair_guidance = repair_guidance or RepairGuidance()
        self.max_iterations = max_iterations

    def run(self, sandbox_result: dict, patch_candidate: dict, artifact_dir: str) -> dict:
        failure = self.failure_analyzer.analyze(
            stderr=str(sandbox_result.get("stderr", "")),
            trace_paths=list(sandbox_result.get("trace_paths", [])),
            test_output_paths=list(sandbox_result.get("test_output_paths", [])),
        )
        guidance = self.repair_guidance.from_failure(failure)
        counterexamples = self.counterexample_generator.generate(failure, selected_tests=sandbox_result.get("selected_tests", []))
        target_symbol = patch_candidate.get("changed_symbols", [failure.get("symbol", "")])
        ephemeral_tests = self.test_mutator.write_ephemeral_tests(
            artifact_dir=artifact_dir,
            target_symbol=str(target_symbol[0] or failure.get("symbol") or "auth.authenticate"),
            counterexamples=counterexamples,
        )
        iterations = [
            {
                "iteration": 1,
                "failure": failure,
                "guidance": guidance,
                "counterexample_count": len(counterexamples),
                "ephemeral_tests": ephemeral_tests,
            }
        ]
        return {
            "iterations": iterations[: self.max_iterations],
            "failure_analysis": failure,
            "repair_guidance": guidance,
            "counterexamples": counterexamples,
            "ephemeral_tests": ephemeral_tests,
        }
