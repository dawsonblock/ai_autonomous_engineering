from __future__ import annotations

import asyncio
from typing import Dict, Iterable, Optional, Set, Tuple

from aae.contracts.tasks import TaskSpec


class TaskScheduler:
    def __init__(self, max_concurrency: int = 4) -> None:
        self.max_concurrency = max_concurrency
        self._queue: asyncio.PriorityQueue[Tuple[int, int, str, TaskSpec]] = asyncio.PriorityQueue()
        self._queued: Set[str] = set()
        self._running: Set[str] = set()
        self._sequence = 0

    def enqueue(self, task: TaskSpec) -> None:
        if task.task_id in self._queued or task.task_id in self._running:
            return
        self._sequence += 1
        self._queue.put_nowait((-task.priority, self._sequence, task.task_id, task))
        self._queued.add(task.task_id)

    def enqueue_many(self, tasks: Iterable[TaskSpec]) -> None:
        for task in tasks:
            self.enqueue(task)

    def has_capacity(self) -> bool:
        return len(self._running) < self.max_concurrency

    def running_count(self) -> int:
        return len(self._running)

    def queue_size(self) -> int:
        return self._queue.qsize()

    def start_next(self) -> Optional[TaskSpec]:
        if not self.has_capacity():
            return None
        try:
            _, _, task_id, task = self._queue.get_nowait()
        except asyncio.QueueEmpty:
            return None
        self._queued.discard(task_id)
        self._running.add(task_id)
        return task

    def complete(self, task_id: str) -> None:
        self._running.discard(task_id)
