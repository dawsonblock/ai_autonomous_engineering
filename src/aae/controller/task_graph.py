from __future__ import annotations

from collections import deque
from typing import Deque, Dict, List, Set, Tuple

from aae.contracts.tasks import TaskSpec, TaskState
from aae.contracts.workflow import WorkflowSpec


class TaskGraph:
    def __init__(self, workflow: WorkflowSpec) -> None:
        self.workflow = workflow
        self.tasks: Dict[str, TaskSpec] = {task.task_id: task for task in workflow.tasks}
        self.states: Dict[str, TaskState] = {
            task.task_id: TaskState.PENDING for task in workflow.tasks
        }
        self.block_reasons: Dict[str, str] = {}
        self.dependents: Dict[str, Set[str]] = {task_id: set() for task_id in self.tasks}

        for task in workflow.tasks:
            for upstream in task.depends_on:
                if upstream not in self.tasks:
                    raise ValueError(
                        "task '%s' depends on unknown task '%s'" % (task.task_id, upstream)
                    )
                self.dependents[upstream].add(task.task_id)

        for task in workflow.tasks:
            if self._is_ready(task):
                self.states[task.task_id] = TaskState.READY

    def get_state(self, task_id: str) -> TaskState:
        return self.states[task_id]

    def is_terminal(self, task_id: str) -> bool:
        return self.states[task_id] in {
            TaskState.SUCCEEDED,
            TaskState.FAILED,
            TaskState.BLOCKED,
            TaskState.CANCELLED,
        }

    def all_terminal(self) -> bool:
        return all(self.is_terminal(task_id) for task_id in self.tasks)

    def ready_tasks(self) -> List[TaskSpec]:
        return [
            self.tasks[task_id]
            for task_id, state in self.states.items()
            if state == TaskState.READY
        ]

    def mark_running(self, task_id: str) -> None:
        self.states[task_id] = TaskState.RUNNING

    def mark_retry_waiting(self, task_id: str) -> None:
        self.states[task_id] = TaskState.RETRY_WAITING

    def mark_ready(self, task_id: str) -> None:
        self.states[task_id] = TaskState.READY

    def mark_succeeded(self, task_id: str) -> Dict[str, List[str]]:
        self.states[task_id] = TaskState.SUCCEEDED
        return self._resolve_dependents(task_id)

    def mark_failed(self, task_id: str, reason: str = "") -> Dict[str, List[str]]:
        self.states[task_id] = TaskState.FAILED
        if reason:
            self.block_reasons[task_id] = reason
        return self._resolve_dependents(task_id)

    def mark_cancelled(self, task_id: str, reason: str = "") -> Dict[str, List[str]]:
        self.states[task_id] = TaskState.CANCELLED
        if reason:
            self.block_reasons[task_id] = reason
        return self._resolve_dependents(task_id)

    def mark_blocked(self, task_id: str, reason: str) -> Dict[str, List[str]]:
        self.states[task_id] = TaskState.BLOCKED
        self.block_reasons[task_id] = reason
        return self._resolve_dependents(task_id)

    def _hard_dependencies(self, task: TaskSpec) -> Set[str]:
        return set(task.depends_on) - set(task.soft_dependencies)

    def _dependencies_terminal(self, task: TaskSpec) -> bool:
        return all(self.is_terminal(dep) for dep in task.depends_on)

    def _is_ready(self, task: TaskSpec) -> bool:
        hard_deps = self._hard_dependencies(task)
        return (
            all(self.states[dep] == TaskState.SUCCEEDED for dep in hard_deps)
            and self._dependencies_terminal(task)
        )

    def _has_failed_hard_dependency(self, task: TaskSpec) -> bool:
        hard_deps = self._hard_dependencies(task)
        return any(
            self.states[dep] in {TaskState.FAILED, TaskState.BLOCKED, TaskState.CANCELLED}
            for dep in hard_deps
        )

    def _resolve_dependents(self, changed_task_id: str) -> Dict[str, List[str]]:
        ready: List[str] = []
        blocked: List[str] = []
        queue: Deque[str] = deque(self.dependents.get(changed_task_id, set()))

        while queue:
            task_id = queue.popleft()
            state = self.states[task_id]
            if state not in {TaskState.PENDING, TaskState.RETRY_WAITING}:
                continue

            task = self.tasks[task_id]
            if self._has_failed_hard_dependency(task):
                self.states[task_id] = TaskState.BLOCKED
                self.block_reasons[task_id] = "hard dependency failed"
                blocked.append(task_id)
                queue.extend(self.dependents.get(task_id, set()))
                continue

            if self._is_ready(task):
                self.states[task_id] = TaskState.READY
                ready.append(task_id)

        return {"ready": ready, "blocked": blocked}
