from __future__ import annotations

import asyncio
from typing import Any, Dict, List, Tuple

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
    ) -> None:
        self.registry = registry
        self.memory = memory
        self.event_bus = event_bus
        self.scheduler = scheduler or TaskScheduler()
        self.retry_policy = retry_policy or RetryPolicy()

    async def run_workflow(self, workflow: WorkflowSpec) -> Dict[str, Any]:
        graph = TaskGraph(workflow)
        workflow_ns = "workflow/%s" % workflow.workflow_id
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

        in_flight: Dict[asyncio.Task[TaskResult], TaskSpec] = {}

        while not graph.all_terminal():
            while self.scheduler.has_capacity():
                next_task = self.scheduler.start_next()
                if next_task is None:
                    break
                graph.mark_running(next_task.task_id)
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
                payload={"final_states": final_states},
            )
        )
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

    def _current_attempt(self, workflow_ns: str, task_id: str) -> int:
        task_results = self.memory.get(workflow_ns, "task_results") or {}
        task_entry = task_results.get(task_id, {})
        return int(task_entry.get("attempt", 0))

    async def _dispatch_task(self, workflow_id: str, task: TaskSpec) -> TaskResult:
        adapter = self.registry.get(task.agent_name)
        workflow_ns = "workflow/%s" % workflow_id
        memory_snapshot = self.memory.snapshot(workflow_ns)
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
