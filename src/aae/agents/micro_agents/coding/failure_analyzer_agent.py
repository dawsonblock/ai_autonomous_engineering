from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent
from aae.test_repair.failure_analyzer import FailureAnalyzer
from aae.test_repair.repair_guidance import RepairGuidance


class FailureAnalyzerAgent(BaseMicroAgent):
    name = "failure_analyzer"

    def __init__(
        self,
        failure_analyzer: FailureAnalyzer | None = None,
        repair_guidance: RepairGuidance | None = None,
    ) -> None:
        self.failure_analyzer = failure_analyzer or FailureAnalyzer()
        self.repair_guidance = repair_guidance or RepairGuidance()

    async def run(self, task, context):
        analysis = self.failure_analyzer.analyze(
            stderr=str(context.get("test_logs", "")),
            trace_paths=list(context.get("trace_paths", [])),
            test_output_paths=list(context.get("test_output_paths", [])),
        )
        guidance = self.repair_guidance.from_failure(analysis)
        failure_type = "logic_error" if "assert" in analysis.get("error", "").lower() or "expected" in analysis.get("error", "").lower() else "execution_error"
        return {
            "failure_type": failure_type,
            "suspected_file": analysis.get("file", ""),
            "reason": analysis.get("error", "") or "No test output available",
            "repair_guidance": guidance,
        }
