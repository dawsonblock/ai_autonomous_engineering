from __future__ import annotations

from collections import deque
from enum import Enum
from typing import Any, Dict, List, Set

from aae.controller.task_graph import TaskGraph


class ActionState(str, Enum):
    PENDING = "pending"
    READY = "ready"
    RUNNING = "running"
    DONE = "done"
    FAILED = "failed"


class ActionGraph:
    """Dependency graph of actions for multi-step workflows."""

    def __init__(self) -> None:
        self.nodes: Dict[str, Dict[str, Any]] = {}
        self.edges: List[tuple[str, str]] = []
        self._dependents: Dict[str, Set[str]] = {}
        self._dependencies: Dict[str, Set[str]] = {}
        self._states: Dict[str, ActionState] = {}

    def add_task(self, task_id: str, metadata: Dict[str, Any] | None = None) -> None:
        self.nodes[task_id] = metadata or {}
        self._dependents.setdefault(task_id, set())
        self._dependencies.setdefault(task_id, set())
        self._states[task_id] = ActionState.PENDING
        if not self._dependencies[task_id]:
            self._states[task_id] = ActionState.READY

    def add_edge(self, from_task: str, to_task: str) -> None:
        self.edges.append((from_task, to_task))
        self._dependents.setdefault(from_task, set()).add(to_task)
        self._dependencies.setdefault(to_task, set()).add(from_task)
        if self._states.get(to_task) == ActionState.READY:
            self._states[to_task] = ActionState.PENDING

    def get_ready(self) -> List[str]:
        return [
            task_id
            for task_id, state in self._states.items()
            if state == ActionState.READY
        ]

    def mark_running(self, task_id: str) -> None:
        self._states[task_id] = ActionState.RUNNING

    def mark_done(self, task_id: str) -> List[str]:
        self._states[task_id] = ActionState.DONE
        newly_ready: List[str] = []
        for dependent in self._dependents.get(task_id, set()):
            if all(
                self._states.get(dep) == ActionState.DONE
                for dep in self._dependencies.get(dependent, set())
            ):
                self._states[dependent] = ActionState.READY
                newly_ready.append(dependent)
        return newly_ready

    def mark_failed(self, task_id: str) -> None:
        self._states[task_id] = ActionState.FAILED

    def get_state(self, task_id: str) -> ActionState:
        return self._states[task_id]

    def all_done(self) -> bool:
        return all(state == ActionState.DONE for state in self._states.values())

    def topological_order(self) -> List[str]:
        in_degree: Dict[str, int] = {task_id: 0 for task_id in self.nodes}
        for _from, to in self.edges:
            in_degree[to] = in_degree.get(to, 0) + 1
        queue: deque[str] = deque(
            task_id for task_id, degree in in_degree.items() if degree == 0
        )
        order: List[str] = []
        while queue:
            task_id = queue.popleft()
            order.append(task_id)
            for dependent in self._dependents.get(task_id, set()):
                in_degree[dependent] -= 1
                if in_degree[dependent] == 0:
                    queue.append(dependent)
        if len(order) != len(self.nodes):
            raise ValueError(
                "Cycle detected in ActionGraph; topological order not possible"
            )
        return order


__all__ = ["ActionGraph", "ActionState", "TaskGraph"]
