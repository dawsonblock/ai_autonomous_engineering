from __future__ import annotations

import asyncio
import json
import os
import shutil
from collections import defaultdict
from pathlib import Path
from typing import Any, DefaultDict, Dict, List

import httpx

from aae.contracts.dashboard import (
    DashboardBenchmarkRunRequest,
    DashboardWorkflowLaunchRequest,
    RuntimeOverrideProfile,
    SystemDiagnostic,
)
from aae.contracts.workflow import EventEnvelope
from aae.evaluation.benchmark_runner import BenchmarkRunner
from aae.runtime.config import SystemConfig
from aae.runtime.system_launcher import build_runtime
from aae.runtime.workflow_presets import research_only, secure_build, security_only, swe_only

from .run_store import RunStore


class RuntimeManager:
    def __init__(self, config_path: str | None = None) -> None:
        self.config_path = config_path or os.getenv("AAE_CONFIG", "configs/system_config.yaml")
        self.controller = None
        self.client = None
        self.config: SystemConfig | None = None
        self.artifacts_dir = Path(".artifacts")
        self.run_store = RunStore(self.artifacts_dir)
        self._started = False
        self._workflow_tasks: Dict[str, asyncio.Task] = {}
        self._global_subscribers: List[asyncio.Queue[dict[str, Any]]] = []
        self._workflow_subscribers: DefaultDict[str, List[asyncio.Queue[dict[str, Any]]]] = defaultdict(list)

    async def start(self) -> None:
        if self._started:
            return
        self.config = SystemConfig.load(self.config_path)
        self.controller, self.client = build_runtime(self.config_path)
        if self.controller.event_bus.logger is not None:
            self.artifacts_dir = Path(self.controller.event_bus.logger.artifacts_dir)
        self.run_store = RunStore(self.artifacts_dir)
        self.run_store.reindex()
        self.controller.event_bus.subscribe("*", self._handle_event)
        await self.controller.event_bus.start()
        self._started = True

    async def close(self) -> None:
        for task in list(self._workflow_tasks.values()):
            if not task.done():
                task.cancel()
        if self._workflow_tasks:
            await asyncio.gather(*self._workflow_tasks.values(), return_exceptions=True)
        if self.controller is not None:
            await self.controller.event_bus.close()
        if self.client is not None:
            await self.client.aclose()
        self._started = False

    async def launch_workflow(self, request: DashboardWorkflowLaunchRequest) -> dict[str, Any]:
        await self.start()
        workflow = self._workflow_from_request(request)
        self.run_store.register_launch(workflow.model_dump(mode="json"), request.model_dump(mode="json"))
        workflow_task = asyncio.create_task(self._execute_workflow(workflow.workflow_id, workflow))
        self._workflow_tasks[workflow.workflow_id] = workflow_task
        detail = self.get_workflow_detail(workflow.workflow_id)
        return detail or {"workflow_id": workflow.workflow_id, "status": "queued"}

    async def rerun_workflow(self, workflow_id: str) -> dict[str, Any]:
        launch_request = self.run_store.get_launch_request(workflow_id)
        if launch_request is None:
            raise KeyError(f"workflow '{workflow_id}' does not have a recorded launch request")
        new_request = DashboardWorkflowLaunchRequest.model_validate({**launch_request, "workflow_id": ""})
        return await self.launch_workflow(new_request)

    async def cancel_workflow(self, workflow_id: str) -> bool:
        await self.start()
        return await self.controller.cancel_workflow(workflow_id)

    def list_workflows(self) -> list[dict[str, Any]]:
        items = {item["workflow_id"]: dict(item) for item in self.run_store.list_workflows()}
        if self.controller is not None:
            for state in self.controller.list_active_workflows():
                entry = items.setdefault(state["workflow_id"], {})
                entry.update(state)
        ordered = list(items.values())
        ordered.sort(key=lambda item: item.get("updated_at", "") or "", reverse=True)
        return ordered

    def get_workflow_detail(self, workflow_id: str) -> dict[str, Any] | None:
        detail = self.run_store.get_workflow_detail(workflow_id)
        if detail is None:
            return None
        if self.controller is not None:
            workflow_ns = f"workflow/{workflow_id}"
            live_memory = self.controller.memory.snapshot(workflow_ns)
            if live_memory:
                detail["memory_snapshot"] = live_memory
            controller_state = self.controller.get_workflow_state(workflow_id)
            if controller_state is not None:
                detail.update(controller_state)
        detail["artifacts"] = {
            "event_log_path": detail.get("event_log_path", ""),
            "patch_provenance_path": detail.get("patch_provenance_path", ""),
            "benchmark_reports": detail.get("benchmark_reports", []),
        }
        detail["planner"] = {
            "planner_actions": detail.get("memory_snapshot", {}).get("planner_actions", []),
            "branch_comparison": detail.get("memory_snapshot", {}).get("branch_comparison", {}),
            "patch_provenance": detail.get("patch_provenance", []),
        }
        return detail

    def subscribe(self, workflow_id: str | None = None) -> asyncio.Queue[dict[str, Any]]:
        queue: asyncio.Queue[dict[str, Any]] = asyncio.Queue(maxsize=256)
        if workflow_id:
            self._workflow_subscribers[workflow_id].append(queue)
        else:
            self._global_subscribers.append(queue)
        return queue

    def unsubscribe(self, queue: asyncio.Queue[dict[str, Any]], workflow_id: str | None = None) -> None:
        targets = self._workflow_subscribers[workflow_id] if workflow_id else self._global_subscribers
        if queue in targets:
            targets.remove(queue)

    async def run_benchmarks(self, request: DashboardBenchmarkRunRequest) -> dict[str, Any]:
        await self.start()
        corpus_path = request.corpus_path or None
        runner = BenchmarkRunner(
            corpus_path=corpus_path,
            artifacts_dir=str(self.artifacts_dir / "evaluation"),
        )
        report = await runner.run()
        self.run_store.record_benchmark(report)
        return report

    def list_benchmark_reports(self) -> list[dict[str, Any]]:
        return self.run_store.list_benchmark_reports()

    def latest_benchmark_report(self) -> dict[str, Any] | None:
        return self.run_store.latest_benchmark_report()

    def get_runtime_overrides(self) -> RuntimeOverrideProfile:
        path = self._override_path()
        if not path.exists():
            return RuntimeOverrideProfile()
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            return RuntimeOverrideProfile()
        return RuntimeOverrideProfile.model_validate(payload)

    def update_runtime_overrides(self, profile: RuntimeOverrideProfile) -> RuntimeOverrideProfile:
        path = self._override_path()
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(profile.model_dump_json(indent=2), encoding="utf-8")
        if self.controller is not None and profile.controller_concurrency:
            self.controller.scheduler.max_concurrency = int(profile.controller_concurrency)
        return profile

    async def diagnostics(self) -> list[SystemDiagnostic]:
        await self.start()
        docker_path = shutil.which("docker")
        docker_available = docker_path is not None
        openai_configured = bool(os.getenv("OPENAI_API_KEY"))
        postgres_configured = bool(os.getenv("AAE_DATABASE_URL"))
        agentfield_status = "unreachable"
        agentfield_summary = self.config.agentfield.base_url if self.config is not None else ""
        if self.client is not None:
            try:
                response = await self.client._client.get(self.client.base_url, timeout=2.0)
                agentfield_status = "reachable" if response.status_code < 500 else "unreachable"
            except Exception:
                agentfield_status = "unreachable"
        sandbox_trust = "strict" if docker_available else "degraded"
        diagnostics = [
            SystemDiagnostic(
                name="runtime",
                status="ok" if self._started else "warn",
                summary="embedded control plane active" if self._started else "runtime not started",
                details={"config_path": self.config_path},
            ),
            SystemDiagnostic(
                name="docker",
                status="ok" if docker_available else "warn",
                summary="docker available" if docker_available else "docker unavailable",
                details={"path": docker_path or "", "default_trust_level": sandbox_trust},
            ),
            SystemDiagnostic(
                name="agentfield",
                status="ok" if agentfield_status == "reachable" else "warn",
                summary=agentfield_summary,
                details={"status": agentfield_status},
            ),
            SystemDiagnostic(
                name="postgres",
                status="ok" if postgres_configured else "warn",
                summary="postgres configured" if postgres_configured else "postgres not configured",
                details={"configured": postgres_configured},
            ),
            SystemDiagnostic(
                name="openai",
                status="ok" if openai_configured else "warn",
                summary="OpenAI configured" if openai_configured else "OpenAI not configured",
                details={"configured": openai_configured},
            ),
            SystemDiagnostic(
                name="event_bus",
                status="ok",
                summary=self.controller.event_bus.transport_mode if self.controller is not None else "memory",
                details={"transport_mode": self.controller.event_bus.transport_mode if self.controller is not None else "memory"},
            ),
            SystemDiagnostic(
                name="artifacts",
                status="ok" if self.artifacts_dir.exists() else "warn",
                summary=str(self.artifacts_dir),
                details={"exists": self.artifacts_dir.exists()},
            ),
        ]
        return diagnostics

    async def _execute_workflow(self, workflow_id: str, workflow) -> None:
        try:
            result = await self.controller.run_workflow(workflow)
            workflow_ns = f"workflow/{workflow_id}"
            self.run_store.attach_result(workflow_id, result, self.controller.memory.snapshot(workflow_ns))
        except Exception as exc:
            workflow_ns = f"workflow/{workflow_id}"
            self.run_store.attach_result(
                workflow_id,
                {"workflow_id": workflow_id, "workflow_type": workflow.workflow_type, "final_states": {}, "error": str(exc)},
                self.controller.memory.snapshot(workflow_ns),
            )
            raise
        finally:
            self._workflow_tasks.pop(workflow_id, None)

    async def _handle_event(self, event: EventEnvelope) -> None:
        payload = event.model_dump(mode="json")
        self.run_store.apply_event(payload)
        await self._broadcast(payload)

    async def _broadcast(self, payload: dict[str, Any]) -> None:
        workflow_id = str(payload.get("workflow_id", ""))
        for queue in list(self._global_subscribers):
            self._offer(queue, payload)
        for queue in list(self._workflow_subscribers.get(workflow_id, [])):
            self._offer(queue, payload)

    def _offer(self, queue: asyncio.Queue[dict[str, Any]], payload: dict[str, Any]) -> None:
        if queue.full():
            try:
                queue.get_nowait()
            except asyncio.QueueEmpty:
                pass
        queue.put_nowait(payload)

    def _workflow_from_request(self, request: DashboardWorkflowLaunchRequest):
        if request.workflow == "research_only":
            return research_only(query=request.query, workflow_id=request.workflow_id or None)
        if request.workflow == "security_only":
            return security_only(repo_url=request.repo_url, workflow_id=request.workflow_id or None)
        if request.workflow == "swe_only":
            return swe_only(goal=request.goal, repo_url=request.repo_url, workflow_id=request.workflow_id or None)
        return secure_build(
            goal=request.goal,
            repo_url=request.repo_url,
            query=request.query or None,
            include_research=request.include_research,
            include_post_audit=request.include_post_audit,
            workflow_id=request.workflow_id or None,
        )

    def _override_path(self) -> Path:
        return self.artifacts_dir / "dashboard" / "runtime_overrides.json"
