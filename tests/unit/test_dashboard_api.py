from __future__ import annotations

import asyncio

from fastapi.testclient import TestClient

from aae.contracts.dashboard import RuntimeOverrideProfile, SystemDiagnostic
from aae.dashboard_api.server import create_app


class FakeRuntimeManager:
    def __init__(self) -> None:
        self.overrides = RuntimeOverrideProfile(controller_concurrency=4)
        self._summary = {
            "workflow_id": "wf-123",
            "workflow_type": "secure_build",
            "status": "running",
            "started_at": "2026-03-10T00:00:00+00:00",
            "updated_at": "2026-03-10T00:00:00+00:00",
            "completed_at": None,
            "metadata": {"goal": "fix auth"},
            "final_states": {},
            "event_count": 1,
            "active_tasks": ["swe_build"],
            "trust_levels": ["degraded"],
            "launch_request": {"workflow": "secure_build", "goal": "fix auth", "repo_url": "/tmp/repo"},
            "events": [{"event_type": "workflow.started", "workflow_id": "wf-123"}],
            "memory_snapshot": {"patch_provenance": []},
            "artifacts": {"event_log_path": "/tmp/events.jsonl"},
            "planner": {"branch_comparison": {}},
        }

    async def start(self) -> None:
        return None

    async def close(self) -> None:
        return None

    async def launch_workflow(self, request):
        self._summary = {**self._summary, "workflow_type": request.workflow, "launch_request": request.model_dump(mode="json")}
        return dict(self._summary)

    async def rerun_workflow(self, workflow_id: str):
        return dict(self._summary)

    async def cancel_workflow(self, workflow_id: str) -> bool:
        return workflow_id == self._summary["workflow_id"]

    def list_workflows(self):
        return [dict(self._summary)]

    def get_workflow_detail(self, workflow_id: str):
        if workflow_id != self._summary["workflow_id"]:
            return None
        return dict(self._summary)

    def subscribe(self, workflow_id: str | None = None):
        queue: asyncio.Queue[dict] = asyncio.Queue()
        queue.put_nowait({"workflow_id": workflow_id or "wf-123", "event_type": "workflow.started"})
        queue.put_nowait({"__close__": True})
        return queue

    def unsubscribe(self, queue, workflow_id: str | None = None) -> None:
        return None

    async def run_benchmarks(self, request):
        return {
            "run_id": "bench-1",
            "metrics": {"strict_fix_rate": 0.0, "raw_fix_rate": 1.0, "degraded_run_count": 2},
            "report_path": "/tmp/benchmark_report.json",
            "markdown_report_path": "/tmp/benchmark_report.md",
        }

    def list_benchmark_reports(self):
        return [
            {
                "run_id": "bench-1",
                "metrics": {"strict_fix_rate": 0.0, "raw_fix_rate": 1.0, "degraded_run_count": 2},
                "report_path": "/tmp/benchmark_report.json",
                "markdown_report_path": "/tmp/benchmark_report.md",
            }
        ]

    def latest_benchmark_report(self):
        return self.list_benchmark_reports()[0]

    def get_runtime_overrides(self):
        return self.overrides

    def update_runtime_overrides(self, profile):
        self.overrides = profile
        return profile

    async def diagnostics(self):
        return [
            SystemDiagnostic(name="docker", status="warn", summary="docker unavailable", details={"default_trust_level": "degraded"}),
            SystemDiagnostic(name="event_bus", status="ok", summary="memory", details={"transport_mode": "memory"}),
        ]


def test_dashboard_workflow_routes_and_settings():
    app = create_app(FakeRuntimeManager())
    client = TestClient(app)

    workflows = client.get("/api/workflows")
    assert workflows.status_code == 200
    assert workflows.json()[0]["workflow_id"] == "wf-123"

    launched = client.post(
        "/api/workflows",
        json={"workflow": "secure_build", "goal": "fix auth", "repo_url": "/tmp/repo"},
    )
    assert launched.status_code == 200
    assert launched.json()["launch_request"]["goal"] == "fix auth"

    detail = client.get("/api/workflows/wf-123")
    assert detail.status_code == 200
    assert detail.json()["summary"]["workflow_id"] == "wf-123"

    cancelled = client.post("/api/workflows/wf-123/cancel")
    assert cancelled.status_code == 200
    assert cancelled.json()["cancelled"] is True

    settings = client.put(
        "/api/settings/runtime-overrides",
        json={"controller_concurrency": 7, "planner": {"beam_width": 3}, "localization": {"stacktrace": 0.3}},
    )
    assert settings.status_code == 200
    assert settings.json()["controller_concurrency"] == 7

    health = client.get("/api/health")
    assert health.status_code == 200
    assert health.json()[0]["name"] == "docker"


def test_dashboard_event_stream_and_benchmarks():
    app = create_app(FakeRuntimeManager())
    client = TestClient(app)

    with client.stream("GET", "/api/events/stream") as response:
        lines = []
        for line in response.iter_lines():
            if line:
                lines.append(line)
            if len(lines) >= 2:
                break
    assert any("workflow.started" in line for line in lines)

    benchmark = client.post("/api/benchmarks/run", json={})
    assert benchmark.status_code == 200
    assert benchmark.json()["metrics"]["raw_fix_rate"] == 1.0
