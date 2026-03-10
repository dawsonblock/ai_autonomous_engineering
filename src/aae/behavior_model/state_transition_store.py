from __future__ import annotations

import json
from pathlib import Path

from aae.contracts.behavior import BehaviorSnapshot, TraceRecord


class StateTransitionStore:
    def __init__(self, base_dir: str) -> None:
        self.base_dir = Path(base_dir)
        self.base_dir.mkdir(parents=True, exist_ok=True)

    def store_snapshot(self, workflow_id: str, snapshot: BehaviorSnapshot) -> dict:
        workflow_dir = self.base_dir / workflow_id
        workflow_dir.mkdir(parents=True, exist_ok=True)
        snapshot_path = workflow_dir / "behavior_snapshot.json"
        snapshot_path.write_text(json.dumps(snapshot.model_dump(mode="json"), indent=2, sort_keys=True), encoding="utf-8")
        traces_path = workflow_dir / "trace_records.jsonl"
        if not traces_path.exists():
            traces_path.write_text("", encoding="utf-8")
        return {
            "snapshot_path": str(snapshot_path),
            "trace_path": str(traces_path),
            "root_path": snapshot.root_path,
            "stats": snapshot.metadata,
        }

    def append_traces(self, workflow_id: str, traces: list[TraceRecord]) -> str:
        workflow_dir = self.base_dir / workflow_id
        workflow_dir.mkdir(parents=True, exist_ok=True)
        trace_path = workflow_dir / "trace_records.jsonl"
        with trace_path.open("a", encoding="utf-8") as handle:
            for trace in traces:
                handle.write(json.dumps(trace.model_dump(mode="json"), sort_keys=True))
                handle.write("\n")
        return str(trace_path)

    def load_snapshot(self, workflow_id: str) -> BehaviorSnapshot | None:
        snapshot_path = self.base_dir / workflow_id / "behavior_snapshot.json"
        if not snapshot_path.exists():
            return None
        return BehaviorSnapshot.model_validate(json.loads(snapshot_path.read_text(encoding="utf-8")))

    def load_traces(self, workflow_id: str) -> list[TraceRecord]:
        trace_path = self.base_dir / workflow_id / "trace_records.jsonl"
        if not trace_path.exists():
            return []
        traces = []
        for line in trace_path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            traces.append(TraceRecord.model_validate(json.loads(line)))
        return traces
