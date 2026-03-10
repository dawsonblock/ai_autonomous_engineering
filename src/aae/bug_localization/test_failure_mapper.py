from __future__ import annotations

from typing import Any, Dict, List

from aae.bug_localization.stack_trace_analyzer import StackTraceAnalyzer
from aae.contracts.localization import FailureEvidence


class TestFailureMapper:
    def __init__(self, stack_trace_analyzer: StackTraceAnalyzer | None = None) -> None:
        self.stack_trace_analyzer = stack_trace_analyzer or StackTraceAnalyzer()

    def map_failures(self, context: Dict[str, Any]) -> List[FailureEvidence]:
        evidence: List[FailureEvidence] = []
        for test_path in context.get("covering_tests", []):
            evidence.append(
                FailureEvidence(
                    source="covering_test",
                    file_path=test_path,
                    weight=0.3,
                )
            )
        for test_path in context.get("impacted_tests", []):
            evidence.append(
                FailureEvidence(
                    source="impacted_test",
                    file_path=test_path,
                    weight=0.5,
                )
            )

        for run in context.get("sandbox_runs", []):
            stderr = run.get("stderr", "")
            evidence.extend(self.stack_trace_analyzer.parse(stderr))
            for path in run.get("selected_tests", []):
                evidence.append(
                    FailureEvidence(
                        source="sandbox_test",
                        file_path=path,
                        weight=0.6 if run.get("returncode", 0) else 0.2,
                        metadata={"command_id": run.get("command_id", "")},
                    )
                )
        for message in context.get("recent_failures", []):
            evidence.extend(self.stack_trace_analyzer.parse(message))
        for trace in context.get("trace_records", []):
            if trace.get("event_type") == "exception":
                evidence.append(
                    FailureEvidence(
                        source="trace_exception",
                        file_path=trace.get("file_path", ""),
                        symbol=trace.get("function", ""),
                        line=int(trace.get("line", 0) or 0),
                        weight=0.8,
                    )
                )
        return evidence
