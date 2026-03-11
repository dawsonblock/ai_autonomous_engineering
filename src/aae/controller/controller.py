from __future__ import annotations

import asyncio
from datetime import datetime, timezone
from typing import Any, Dict, List

from aae.contracts.results import TaskError, TaskResult, TaskResultStatus
from aae.contracts.tasks import TaskSpec, TaskState
from aae.contracts.workflow import EventEnvelope, WorkflowSpec
from aae.controller.agent_registry import AgentRegistry
from aae.controller.retry_policy import RetryPolicy
from aae.controller.task_graph import TaskGraph
from aae.controller.task_scheduler import TaskScheduler
from aae.events.event_bus import EventBus
from aae.memory.base import MemoryStore


class WorkflowController:
    def __init__(
        self,
        registry: AgentRegistry,
        memory: MemoryStore,
        event_bus: EventBus,
        scheduler: TaskScheduler | None = None,
        retry_policy: RetryPolicy | None = None,
        task_preparer: Any | None = None,
    ) -> None:
        self.registry = registry
        self.memory = memory
        self.event_bus = event_bus
        self.scheduler = scheduler or TaskScheduler()
        self.retry_policy = retry_policy or RetryPolicy()
        self.task_preparer = task_preparer
        self._active_workflows: Dict[str, Dict[str, Any]] = {}
        self._workflow_cancel_events: Dict[str, asyncio.Event] = {}

    async def cancel_workflow(self, workflow_id: str) -> bool:
        cancel_event = self._workflow_cancel_events.get(workflow_id)
        if cancel_event is None:
            return False
        cancel_event.set()
        state = self._active_workflows.get(workflow_id)
        if state is not None:
            state["status"] = "cancel_requested"
            state["updated_at"] = self._timestamp()
        return True

    def list_active_workflows(self) -> List[Dict[str, Any]]:
        return [dict(item) for item in self._active_workflows.values() if item.get("status") in {"running", "cancel_requested"}]

    def get_workflow_state(self, workflow_id: str) -> Dict[str, Any] | None:
        state = self._active_workflows.get(workflow_id)
        return dict(state) if state is not None else None

    async def run_workflow(self, workflow: WorkflowSpec) -> Dict[str, Any]:
        graph = TaskGraph(workflow)
        workflow_ns = "workflow/%s" % workflow.workflow_id
        cancel_event = asyncio.Event()
        self._workflow_cancel_events[workflow.workflow_id] = cancel_event
        self._active_workflows[workflow.workflow_id] = {
            "workflow_id": workflow.workflow_id,
            "workflow_type": workflow.workflow_type,
            "status": "running",
            "started_at": self._timestamp(),
            "updated_at": self._timestamp(),
            "completed_at": None,
            "metadata": dict(workflow.metadata),
            "final_states": {},
            "active_tasks": [],
        }
        self.memory.put(workflow_ns, "metadata", workflow.metadata)
        self.memory.put(workflow_ns, "workflow_type", workflow.workflow_type)
        self.memory.put(workflow_ns, "task_order", [task.task_id for task in workflow.tasks])

        await self.event_bus.publish(
            EventEnvelope(
                event_type="workflow.started",
                workflow_id=workflow.workflow_id,
                source="controller",
                payload={"workflow_type": workflow.workflow_type},
            )
        )

        self.scheduler.enqueue_many(graph.ready_tasks())
        await self._emit_ready_events(workflow.workflow_id, graph.ready_tasks())
        self._refresh_workflow_state(workflow, graph, status="running")

        in_flight: Dict[asyncio.Task[TaskResult], TaskSpec] = {}

        while not graph.all_terminal():
            if cancel_event.is_set():
                await self._cancel_remaining_tasks(workflow, graph, in_flight)
                break
            while self.scheduler.has_capacity():
                next_task = self.scheduler.start_next()
                if next_task is None:
                    break
                graph.mark_running(next_task.task_id)
                self._refresh_workflow_state(workflow, graph, status="running")
                await self.event_bus.publish(
                    EventEnvelope(
                        event_type="task.dispatched",
                        workflow_id=workflow.workflow_id,
                        task_id=next_task.task_id,
                        source="controller",
                        payload={"agent_name": next_task.agent_name, "attempt": self._current_attempt(workflow_ns, next_task.task_id) + 1},
                    )
                )
                execution = asyncio.create_task(
                    self._dispatch_task(workflow.workflow_id, next_task)
                )
                in_flight[execution] = next_task

            if not in_flight:
                await self._block_unrunnable_tasks(workflow.workflow_id, graph)
                continue

            done, _ = await asyncio.wait(
                in_flight.keys(), return_when=asyncio.FIRST_COMPLETED
            )
            for future in done:
                task_spec = in_flight.pop(future)
                self.scheduler.complete(task_spec.task_id)
                try:
                    result = future.result()
                except asyncio.CancelledError:
                    result = TaskResult(
                        task_id=task_spec.task_id,
                        status=TaskResultStatus.FAILED,
                        error=TaskError(
                            message="workflow cancellation requested",
                            error_type="cancelled",
                            transient=False,
                        ),
                    )
                except Exception as exc:
                    result = TaskResult(
                        task_id=task_spec.task_id,
                        status=TaskResultStatus.FAILED,
                        error=TaskError(
                            message=str(exc),
                            error_type="controller",
                            transient=False,
                        ),
                    )
                await self._process_result(workflow, graph, task_spec, result)
                self._refresh_workflow_state(workflow, graph, status="cancel_requested" if cancel_event.is_set() else "running")

        final_states = {
            task_id: graph.get_state(task_id).value for task_id in graph.tasks
        }
        self.memory.put(workflow_ns, "final_states", final_states)
        summary = {
            "workflow_id": workflow.workflow_id,
            "workflow_type": workflow.workflow_type,
            "final_states": final_states,
            "results": self.memory.snapshot(workflow_ns).get("task_results", {}),
        }
        await self.event_bus.publish(
            EventEnvelope(
                event_type="workflow.completed",
                workflow_id=workflow.workflow_id,
                source="controller",
                payload={"final_states": final_states, "cancelled": cancel_event.is_set()},
            )
        )
        self._refresh_workflow_state(
            workflow,
            graph,
            status="cancelled" if cancel_event.is_set() else "completed",
            final_states=final_states,
            completed=True,
        )
        self._workflow_cancel_events.pop(workflow.workflow_id, None)
        return summary

    async def _block_unrunnable_tasks(self, workflow_id: str, graph: TaskGraph) -> None:
        blocked_any = False
        for task_id, state in graph.states.items():
            if state in {TaskState.PENDING, TaskState.RETRY_WAITING}:
                graph.mark_blocked(task_id, "no runnable tasks remain")
                blocked_any = True
                await self.event_bus.publish(
                    EventEnvelope(
                        event_type="task.blocked",
                        workflow_id=workflow_id,
                        task_id=task_id,
                        source="controller",
                        payload={"reason": "no runnable tasks remain"},
                    )
                )
        if not blocked_any:
            return

    async def _emit_ready_events(self, workflow_id: str, tasks: List[TaskSpec]) -> None:
        for task in tasks:
            await self.event_bus.publish(
                EventEnvelope(
                    event_type="task.ready",
                    workflow_id=workflow_id,
                    task_id=task.task_id,
                    source="controller",
                    payload={"agent_name": task.agent_name},
                )
            )

    async def _cancel_remaining_tasks(
        self,
        workflow: WorkflowSpec,
        graph: TaskGraph,
        in_flight: Dict[asyncio.Task[TaskResult], TaskSpec],
    ) -> None:
        cancellation_reason = "workflow cancellation requested"
        for future in list(in_flight):
            future.cancel()
        if in_flight:
            await asyncio.gather(*in_flight.keys(), return_exceptions=True)
        for future, task_spec in list(in_flight.items()):
            self.scheduler.complete(task_spec.task_id)
            if graph.get_state(task_spec.task_id) == TaskState.RUNNING:
                graph.mark_cancelled(task_spec.task_id, cancellation_reason)
                await self.event_bus.publish(
                    EventEnvelope(
                        event_type="task.blocked",
                        workflow_id=workflow.workflow_id,
                        task_id=task_spec.task_id,
                        source="controller",
                        payload={"reason": cancellation_reason, "state": TaskState.CANCELLED.value},
                    )
                )
        in_flight.clear()
        for task_id, state in list(graph.states.items()):
            if state in {TaskState.PENDING, TaskState.READY, TaskState.RETRY_WAITING}:
                graph.mark_cancelled(task_id, cancellation_reason)
                await self.event_bus.publish(
                    EventEnvelope(
                        event_type="task.blocked",
                        workflow_id=workflow.workflow_id,
                        task_id=task_id,
                        source="controller",
                        payload={"reason": cancellation_reason, "state": TaskState.CANCELLED.value},
                    )
                )
        self._refresh_workflow_state(workflow, graph, status="cancelled")

    def _current_attempt(self, workflow_ns: str, task_id: str) -> int:
        task_results = self.memory.get(workflow_ns, "task_results") or {}
        task_entry = task_results.get(task_id, {})
        return int(task_entry.get("attempt", 0))

    async def _dispatch_task(self, workflow_id: str, task: TaskSpec) -> TaskResult:
        workflow_ns = "workflow/%s" % workflow_id
        memory_snapshot = self.memory.snapshot(workflow_ns)
        if self.task_preparer is not None:
            task = await self.task_preparer.prepare(workflow_id, task, memory_snapshot)
            memory_snapshot = self.memory.snapshot(workflow_ns)
        adapter = self.registry.get(task.agent_name)
        return await adapter.execute(task, memory_snapshot)

    async def _process_result(
        self,
        workflow: WorkflowSpec,
        graph: TaskGraph,
        task_spec: TaskSpec,
        result: TaskResult,
    ) -> None:
        workflow_ns = "workflow/%s" % workflow.workflow_id
        task_ns = "task/%s" % task_spec.task_id
        agent_ns = "agent/%s" % task_spec.agent_name

        result_data = result.model_dump(mode="json")
        task_results = self.memory.get(workflow_ns, "task_results") or {}
        task_results[task_spec.task_id] = result_data
        self.memory.put(workflow_ns, "task_results", task_results)
        self.memory.put(task_ns, "result", result_data)
        self.memory.append(agent_ns, "runs", result_data)

        self.memory.put(workflow_ns, "last_updated_task", task_spec.task_id)
        await self.event_bus.publish(
            EventEnvelope(
                event_type="memory.updated",
                workflow_id=workflow.workflow_id,
                task_id=task_spec.task_id,
                source="controller",
                payload={"namespace": workflow_ns, "key": "task_results/%s" % task_spec.task_id},
            )
        )

        if result.status == TaskResultStatus.SUCCEEDED:
            updates = graph.mark_succeeded(task_spec.task_id)
            await self.event_bus.publish(
                EventEnvelope(
                    event_type="task.succeeded",
                    workflow_id=workflow.workflow_id,
                    task_id=task_spec.task_id,
                    source="controller",
                    payload={"attempt": result.attempt},
                )
            )
            await self._emit_domain_events(workflow.workflow_id, task_spec.task_id, result)
            newly_ready = [graph.tasks[task_id] for task_id in updates["ready"]]
            self.scheduler.enqueue_many(newly_ready)
            await self._emit_ready_events(workflow.workflow_id, newly_ready)
            for blocked_id in updates["blocked"]:
                await self.event_bus.publish(
                    EventEnvelope(
                        event_type="task.blocked",
                        workflow_id=workflow.workflow_id,
                        task_id=blocked_id,
                        source="controller",
                        payload={"reason": graph.block_reasons.get(blocked_id, "")},
                    )
                )
            return

        decision = self.retry_policy.evaluate(task_spec, result)
        if decision.should_retry:
            graph.mark_retry_waiting(task_spec.task_id)
            await self.event_bus.publish(
                EventEnvelope(
                    event_type="task.retry_scheduled",
                    workflow_id=workflow.workflow_id,
                    task_id=task_spec.task_id,
                    source="controller",
                    payload={"delay_s": decision.delay_s, "attempt": result.attempt + 1},
                )
            )
            await asyncio.sleep(decision.delay_s)
            graph.mark_ready(task_spec.task_id)
            self.scheduler.enqueue(task_spec)
            await self._emit_ready_events(workflow.workflow_id, [task_spec])
            return

        updates = graph.mark_failed(
            task_spec.task_id,
            reason=result.error.message if result.error else "task failed",
        )
        await self.event_bus.publish(
            EventEnvelope(
                event_type="task.failed",
                workflow_id=workflow.workflow_id,
                task_id=task_spec.task_id,
                source="controller",
                payload={
                    "attempt": result.attempt,
                    "error": result.error.model_dump() if result.error else {},
                },
            )
        )
        newly_ready = [graph.tasks[task_id] for task_id in updates["ready"]]
        self.scheduler.enqueue_many(newly_ready)
        await self._emit_ready_events(workflow.workflow_id, newly_ready)
        for blocked_id in updates["blocked"]:
            await self.event_bus.publish(
                EventEnvelope(
                    event_type="task.blocked",
                    workflow_id=workflow.workflow_id,
                    task_id=blocked_id,
                    source="controller",
                    payload={"reason": graph.block_reasons.get(blocked_id, "")},
                )
            )

    def _refresh_workflow_state(
        self,
        workflow: WorkflowSpec,
        graph: TaskGraph,
        status: str,
        final_states: Dict[str, str] | None = None,
        completed: bool = False,
    ) -> None:
        self._active_workflows[workflow.workflow_id] = {
            **self._active_workflows.get(workflow.workflow_id, {}),
            "workflow_id": workflow.workflow_id,
            "workflow_type": workflow.workflow_type,
            "status": status,
            "updated_at": self._timestamp(),
            "completed_at": self._timestamp() if completed else self._active_workflows.get(workflow.workflow_id, {}).get("completed_at"),
            "metadata": dict(workflow.metadata),
            "active_tasks": [task_id for task_id, state in graph.states.items() if state in {TaskState.READY, TaskState.RUNNING, TaskState.RETRY_WAITING}],
            "final_states": final_states or {task_id: state.value for task_id, state in graph.states.items()},
        }

    def _timestamp(self) -> datetime:
        return datetime.now(timezone.utc)

    async def _emit_domain_events(
        self, workflow_id: str, task_id: str, result: TaskResult
    ) -> None:
        events = result.normalized_output.get("events", [])
        for event in events:
            await self.event_bus.publish(
                EventEnvelope(
                    event_type=event["event_type"],
                    workflow_id=workflow_id,
                    task_id=task_id,
                    source=event.get("source", "adapter"),
                    payload=event.get("payload", {}),
                )
            )
