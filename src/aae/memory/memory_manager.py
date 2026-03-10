from __future__ import annotations

from aae.memory.base import MemoryStore
from aae.memory.graph_memory import GraphMemory
from aae.memory.trajectory_memory import TrajectoryMemory
from aae.memory.vector_memory import VectorMemory


class MemoryManager:
    def __init__(
        self,
        workflow_memory: MemoryStore,
        graph_memory: GraphMemory | None = None,
        trajectory_memory: TrajectoryMemory | None = None,
        vector_memory: VectorMemory | None = None,
    ) -> None:
        self.workflow_memory = workflow_memory
        self.graph_memory = graph_memory or GraphMemory()
        self.trajectory_memory = trajectory_memory or TrajectoryMemory()
        self.vector_memory = vector_memory or VectorMemory()

    def workflow_snapshot(self, namespace: str) -> dict:
        return self.workflow_memory.snapshot(namespace)
