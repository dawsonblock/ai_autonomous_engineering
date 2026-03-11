from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List

from aae.contracts.workflow import EventEnvelope


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


class RunStore:
    def __init__(self, artifacts_dir: str | Path) -> None:
        self.artifacts_dir = Path(artifacts_dir)
        self.dashboard_dir = self.artifacts_dir / "dashboard"
        self.runs_dir = self.dashboard_dir / "runs"
        self.state_dir = self.dashboard_dir / "workflow_state"
        self._workflows: Dict[str, Dict[str, Any]] = {}
        self._benchmark_reports: List[Dict[str, Any]] = []

    def reindex(self) -> None:
        self.runs_dir.mkdir(parents=True, exist_ok=True)
        self.state_dir.mkdir(parents=True, exist_ok=True)
        self._workflows = {}
        self._benchmark_reports = []
        self._load_launch_manifests()
        self._load_state_files()
        for event_log in sorted(self.artifacts_dir.rglob("events/*.jsonl")):
            self._index_event_log(event_log)
        for provenance_log in sorted(self.artifacts_dir.rglob("patch_provenance/*.jsonl")):
            self._attach_patch_provenance(provenance_log)
        for report_path in sorted(self.artifacts_dir.rglob("benchmark_report.json")):
            self._attach_benchmark_report(report_path)
        self._benchmark_reports.sort(key=lambda item: item.get("updated_at", ""), reverse=True)

    def register_launch(self, workflow: dict[str, Any], launch_request: dict[str, Any]) -> None:
        workflow_id = str(workflow["workflow_id"])
        payload = {
            "workflow_id": workflow_id,
            "workflow_type": workflow.get("workflow_type", ""),
            "metadata": dict(workflow.get("metadata", {})),
            "launch_request": dict(launch_request),
            "created_at": _now_iso(),
        }
        path = self.runs_dir / f"{workflow_id}.json"
        path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
        summary = self._workflow_entry(workflow_id)
        summary.update(
            {
                "workflow_type": payload["workflow_type"],
                "metadata": payload["metadata"],
                "launch_request": payload["launch_request"],
                "status": summary.get("status", "queued"),
                "updated_at": _now_iso(),
            }
        )

    def attach_result(self, workflow_id: str, summary: dict[str, Any], memory_snapshot: dict[str, Any]) -> None:
        detail = {
            "workflow_id": workflow_id,
            "summary": summary,
            "memory_snapshot": memory_snapshot,
            "updated_at": _now_iso(),
        }
        path = self.state_dir / f"{workflow_id}.json"
        path.write_text(json.dumps(detail, indent=2, sort_keys=True), encoding="utf-8")
        entry = self._workflow_entry(workflow_id)
        entry["memory_snapshot"] = memory_snapshot
        entry["result_summary"] = summary
        entry["updated_at"] = detail["updated_at"]

    def record_benchmark(self, report: dict[str, Any]) -> None:
        run_id = str(report.get("run_id", ""))
        existing = next((item for item in self._benchmark_reports if item.get("run_id") == run_id), None)
        entry = {
            "run_id": run_id,
            "metrics": dict(report.get("metrics", {})),
            "report_path": report.get("report_path", ""),
            "markdown_report_path": report.get("markdown_report_path", ""),
            "updated_at": _now_iso(),
            "payload": report,
        }
        if existing is None:
            self._benchmark_reports.append(entry)
        else:
            existing.update(entry)
        self._benchmark_reports.sort(key=lambda item: item.get("updated_at", ""), reverse=True)

    def apply_event(self, event: EventEnvelope | dict[str, Any]) -> None:
        envelope = event if isinstance(event, EventEnvelope) else EventEnvelope.model_validate(event)
        entry = self._workflow_entry(envelope.workflow_id)
        entry["event_count"] = int(entry.get("event_count", 0)) + 1
        entry["updated_at"] = envelope.timestamp.isoformat()
        entry["event_log_path"] = str(self.artifacts_dir / "events" / f"{envelope.workflow_id}.jsonl")

        if envelope.event_type == "workflow.started":
            entry["workflow_type"] = envelope.payload.get("workflow_type", entry.get("workflow_type", ""))
            entry["status"] = "running"
            entry["started_at"] = envelope.timestamp.isoformat()
        elif envelope.event_type == "workflow.completed":
            entry["status"] = "cancelled" if envelope.payload.get("cancelled") else "completed"
            entry["completed_at"] = envelope.timestamp.isoformat()
            entry["final_states"] = dict(envelope.payload.get("final_states", {}))
            entry["active_tasks"] = []
        elif envelope.event_type == "task.dispatched" and envelope.task_id:
            active = list(entry.get("active_tasks", []))
            if envelope.task_id not in active:
                active.append(envelope.task_id)
            entry["active_tasks"] = active
        elif envelope.event_type in {"task.succeeded", "task.failed", "task.blocked"} and envelope.task_id:
            entry["active_tasks"] = [task_id for task_id in entry.get("active_tasks", []) if task_id != envelope.task_id]

    def list_workflows(self) -> List[Dict[str, Any]]:
        items = list(self._workflows.values())
        items.sort(key=lambda item: item.get("updated_at", ""), reverse=True)
        return items

    def get_workflow_detail(self, workflow_id: str) -> Dict[str, Any] | None:
        entry = self._workflows.get(workflow_id)
        if entry is None:
            return None
        detail = dict(entry)
        detail["events"] = self._load_events(workflow_id)
        detail["patch_provenance"] = self._load_patch_provenance(workflow_id)
        detail["benchmark_reports"] = self._workflow_benchmark_reports(workflow_id)
        detail["memory_snapshot"] = detail.get("memory_snapshot", self._load_memory_snapshot(workflow_id))
        return detail

    def get_launch_request(self, workflow_id: str) -> Dict[str, Any] | None:
        entry = self._workflows.get(workflow_id)
        launch_request = entry.get("launch_request") if entry else None
        if isinstance(launch_request, dict):
            return dict(launch_request)
        return None

    def list_benchmark_reports(self) -> List[Dict[str, Any]]:
        return list(self._benchmark_reports)

    def latest_benchmark_report(self) -> Dict[str, Any] | None:
        return self._benchmark_reports[0] if self._benchmark_reports else None

    def _workflow_entry(self, workflow_id: str) -> Dict[str, Any]:
        return self._workflows.setdefault(
            workflow_id,
            {
                "workflow_id": workflow_id,
                "workflow_type": "",
                "status": "pending",
                "started_at": None,
                "updated_at": None,
                "completed_at": None,
                "metadata": {},
                "final_states": {},
                "event_count": 0,
                "active_tasks": [],
                "trust_levels": [],
            },
        )

    def _load_launch_manifests(self) -> None:
        for path in sorted(self.runs_dir.glob("*.json")):
            try:
                payload = json.loads(path.read_text(encoding="utf-8"))
            except json.JSONDecodeError:
                continue
            entry = self._workflow_entry(str(payload.get("workflow_id", path.stem)))
            entry["workflow_type"] = payload.get("workflow_type", entry.get("workflow_type", ""))
            entry["metadata"] = dict(payload.get("metadata", {}))
            entry["launch_request"] = dict(payload.get("launch_request", {}))
            entry["started_at"] = entry.get("started_at") or payload.get("created_at")
            entry["updated_at"] = entry.get("updated_at") or payload.get("created_at")

    def _load_state_files(self) -> None:
        for path in sorted(self.state_dir.glob("*.json")):
            try:
                payload = json.loads(path.read_text(encoding="utf-8"))
            except json.JSONDecodeError:
                continue
            workflow_id = str(payload.get("workflow_id", path.stem))
            entry = self._workflow_entry(workflow_id)
            entry["memory_snapshot"] = payload.get("memory_snapshot", {})
            result_summary = payload.get("summary", {})
            entry["result_summary"] = result_summary
            entry["updated_at"] = payload.get("updated_at", entry.get("updated_at"))
            if isinstance(result_summary, dict):
                entry["final_states"] = dict(result_summary.get("final_states", entry.get("final_states", {})))

    def _index_event_log(self, path: Path) -> None:
        with path.open("r", encoding="utf-8") as handle:
            for line in handle:
                if not line.strip():
                    continue
                try:
                    payload = json.loads(line)
                except json.JSONDecodeError:
                    continue
                self.apply_event(payload)
                workflow_id = str(payload.get("workflow_id", ""))
                if workflow_id:
                    self._workflow_entry(workflow_id)["event_log_path"] = str(path)

    def _attach_patch_provenance(self, path: Path) -> None:
        workflow_id = path.stem
        entry = self._workflow_entry(workflow_id)
        entry["patch_provenance_path"] = str(path)

    def _attach_benchmark_report(self, path: Path) -> None:
        try:
            report = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            return
        self._benchmark_reports.append(
            {
                "run_id": report.get("run_id", path.stem),
                "metrics": dict(report.get("metrics", {})),
                "report_path": str(path),
                "markdown_report_path": str(path.with_suffix(".md")),
                "updated_at": datetime.fromtimestamp(path.stat().st_mtime, tz=timezone.utc).isoformat(),
                "payload": report,
            }
        )

    def _load_events(self, workflow_id: str) -> List[Dict[str, Any]]:
        entry = self._workflows.get(workflow_id, {})
        path = entry.get("event_log_path")
        if not path:
            return []
        event_path = Path(path)
        if not event_path.exists():
            return []
        events = []
        with event_path.open("r", encoding="utf-8") as handle:
            for line in handle:
                if not line.strip():
                    continue
                try:
                    events.append(json.loads(line))
                except json.JSONDecodeError:
                    continue
        return events

    def _load_patch_provenance(self, workflow_id: str) -> List[Dict[str, Any]]:
        entry = self._workflows.get(workflow_id, {})
        path = entry.get("patch_provenance_path")
        if not path:
            return []
        provenance_path = Path(path)
        if not provenance_path.exists():
            return []
        records = []
        with provenance_path.open("r", encoding="utf-8") as handle:
            for line in handle:
                if not line.strip():
                    continue
                try:
                    records.append(json.loads(line))
                except json.JSONDecodeError:
                    continue
        return records

    def _load_memory_snapshot(self, workflow_id: str) -> Dict[str, Any]:
        path = self.state_dir / f"{workflow_id}.json"
        if not path.exists():
            return {}
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            return {}
        return dict(payload.get("memory_snapshot", {}))

    def _workflow_benchmark_reports(self, workflow_id: str) -> List[Dict[str, Any]]:
        results = []
        for report in self._benchmark_reports:
            payload = report.get("payload", {})
            matching = [record for record in payload.get("records", []) if record.get("case_id") == workflow_id or record.get("workflow_id") == workflow_id]
            if matching:
                results.append({**report, "records": matching})
        return results
