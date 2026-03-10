from __future__ import annotations

import atexit
import json
import os
import sys
import threading
import time
import uuid
from pathlib import Path


def install_if_enabled() -> None:
    output_path = os.getenv("AAE_TRACE_OUTPUT", "").strip()
    if not output_path:
        return
    collector = TraceCollector(
        output_path=output_path,
        filter_root=os.getenv("AAE_TRACE_FILTER_ROOT", "").strip(),
        command_id=os.getenv("AAE_TRACE_COMMAND_ID", "").strip(),
        test_id=os.getenv("PYTEST_CURRENT_TEST", "").split(" ", 1)[0],
    )
    collector.install()


class TraceCollector:
    def __init__(self, output_path: str, filter_root: str = "", command_id: str = "", test_id: str = "") -> None:
        self.output_path = Path(output_path)
        self.filter_root = str(Path(filter_root).resolve()) if filter_root else ""
        self.command_id = command_id or "cmd-%s" % uuid.uuid4().hex[:8]
        self.test_id = test_id
        self.records: list[dict] = []
        self._call_stack = threading.local()
        self._installed = False

    def install(self) -> None:
        if self._installed:
            return
        self.output_path.parent.mkdir(parents=True, exist_ok=True)
        sys.settrace(self._trace)
        threading.settrace(self._trace)
        atexit.register(self.flush)
        self._installed = True

    def flush(self) -> None:
        if not self.records:
            return
        with self.output_path.open("a", encoding="utf-8") as handle:
            for record in self.records:
                handle.write(json.dumps(record, sort_keys=True))
                handle.write("\n")
        self.records.clear()

    def _trace(self, frame, event, arg):  # noqa: ANN001
        if event not in {"call", "return", "exception"}:
            return self._trace
        file_path = str(Path(frame.f_code.co_filename).resolve())
        if self.filter_root and not file_path.startswith(self.filter_root):
            return self._trace
        call_stack = getattr(self._call_stack, "stack", [])
        call_id = "call-%s" % uuid.uuid4().hex[:10]
        parent_call_id = call_stack[-1] if call_stack else ""
        if event == "call":
            call_stack.append(call_id)
            self._call_stack.stack = call_stack
        elif event in {"return", "exception"} and call_stack:
            call_id = call_stack.pop()
            self._call_stack.stack = call_stack
        self.records.append(
            {
                "event_type": event,
                "function": frame.f_code.co_name,
                "file_path": file_path,
                "line": int(frame.f_lineno or 0),
                "command_id": self.command_id,
                "test_id": self.test_id,
                "call_id": call_id,
                "parent_call_id": parent_call_id,
                "args_summary": _safe_repr(frame.f_locals),
                "result_summary": _safe_repr(arg) if event == "return" else "",
                "exception_type": _exception_name(arg) if event == "exception" else "",
                "timestamp": str(time.time()),
                "metadata": {},
            }
        )
        if len(self.records) >= 1000:
            self.flush()
        return self._trace


def _safe_repr(value) -> str:  # noqa: ANN001
    try:
        text = repr(value)
    except Exception:  # pragma: no cover - defensive
        return "<unreprable>"
    return text[:400]


def _exception_name(value) -> str:  # noqa: ANN001
    if isinstance(value, tuple) and value:
        exc_type = value[0]
        return getattr(exc_type, "__name__", str(exc_type))
    return ""
