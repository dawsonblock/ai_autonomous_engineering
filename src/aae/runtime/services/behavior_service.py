from __future__ import annotations

import json
from pathlib import Path

from aae.behavior_model.behavior_query_engine import BehaviorQueryEngine
from aae.behavior_model.state_graph_builder import StateGraphBuilder
from aae.behavior_model.state_transition_store import StateTransitionStore
from aae.contracts.behavior import TraceRecord


class BehaviorService:
    def __init__(
        self,
        artifacts_dir: str,
        behavior_builder: StateGraphBuilder | None = None,
        behavior_store: StateTransitionStore | None = None,
    ) -> None:
        self.behavior_builder = behavior_builder or StateGraphBuilder()
        self.behavior_store = behavior_store or StateTransitionStore(base_dir=str(Path(artifacts_dir) / "memory" / "behavior"))

    def ensure_behavior(self, workflow_id: str, repo_path: str, graph_snapshot, behavior_model: dict | None = None):
        if behavior_model is None:
            behavior_snapshot = self.behavior_builder.build(repo_path=repo_path, graph_snapshot=graph_snapshot)
            behavior_model = self.behavior_store.store_snapshot(workflow_id, behavior_snapshot)
        snapshot = self.behavior_store.load_snapshot(workflow_id)
        engine = BehaviorQueryEngine(snapshot) if snapshot is not None else None
        traces = [trace.model_dump(mode="json") for trace in self.behavior_store.load_traces(workflow_id)]
        return behavior_model, engine, traces

    def build_context(self, behavior_engine: BehaviorQueryEngine | None, graph_context: dict) -> dict:
        if behavior_engine is None:
            return {}
        candidate_symbols = graph_context.get("candidate_symbols", [])
        suspicious_files = behavior_engine.suspicious_files(candidate_symbols).items
        causal_paths = []
        for symbol in candidate_symbols[:3]:
            causal_paths.extend(item["path"] for item in behavior_engine.causal_path(symbol).items[:3])
        return {
            "suspicious_files": suspicious_files,
            "causal_paths": causal_paths[:9],
            "trace_overlap": behavior_engine.trace_overlap(candidate_symbols).items[:8],
        }

    def append_trace_files(self, workflow_id: str, trace_paths: list[str]) -> list[dict]:
        appended = []
        for trace_path in trace_paths:
            for trace in self._load_trace_records(trace_path):
                appended.append(trace)
        if appended:
            self.behavior_store.append_traces(workflow_id, appended)
        return [trace.model_dump(mode="json") for trace in appended]

    def _load_trace_records(self, trace_path: str) -> list[TraceRecord]:
        path = Path(trace_path)
        if not path.exists():
            return []
        records = []
        for line in path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            try:
                records.append(TraceRecord.model_validate(json.loads(line)))
            except json.JSONDecodeError:
                continue
        return records
