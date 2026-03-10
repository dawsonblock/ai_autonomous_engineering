from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent


class FailureAnalyzerAgent(BaseMicroAgent):
    name = "failure_analyzer"

    async def run(self, task, context):
        logs = str(context.get("test_logs", ""))
        suspected_file = ""
        for candidate in context.get("changed_files", []):
            if candidate in logs:
                suspected_file = candidate
                break
        failure_type = "logic_error" if "assert" in logs.lower() or "expected" in logs.lower() else "execution_error"
        return {
            "failure_type": failure_type,
            "suspected_file": suspected_file,
            "reason": logs[:240] or "No test output available",
        }
