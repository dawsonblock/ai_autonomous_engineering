from __future__ import annotations

import json
from pathlib import Path

from aae.bug_localization.stack_trace_analyzer import StackTraceAnalyzer


class FailureAnalyzer:
    def __init__(self, stack_trace_analyzer: StackTraceAnalyzer | None = None) -> None:
        self.stack_trace_analyzer = stack_trace_analyzer or StackTraceAnalyzer()

    def analyze(self, stderr: str = "", trace_paths: list[str] | None = None, test_output_paths: list[str] | None = None) -> dict:
        trace_paths = trace_paths or []
        test_output_paths = test_output_paths or []
        stack_items = self.stack_trace_analyzer.parse(stderr)
        trace_events = self._load_trace_events(trace_paths)
        exception_event = next((item for item in trace_events if item.get("event_type") == "exception"), {})
        failure = {
            "file": exception_event.get("file_path") or (stack_items[0].file_path if stack_items else ""),
            "line": int(exception_event.get("line", 0) or (stack_items[0].line if stack_items else 0)),
            "error": exception_event.get("exception_type") or (stack_items[0].metadata.get("raw", "") if stack_items else stderr[:240]),
            "symbol": exception_event.get("function") or (stack_items[0].symbol if stack_items else ""),
            "evidence_sources": ["trace_exception"] if exception_event else (["stack_trace"] if stack_items else ["stderr"]),
            "trace_event_count": len(trace_events),
            "test_outputs": [str(path) for path in test_output_paths],
        }
        return failure

    def _load_trace_events(self, trace_paths: list[str]) -> list[dict]:
        events = []
        for trace_path in trace_paths:
            path = Path(trace_path)
            if not path.exists():
                continue
            for line in path.read_text(encoding="utf-8").splitlines():
                if not line.strip():
                    continue
                try:
                    events.append(json.loads(line))
                except json.JSONDecodeError:
                    continue
        return events
