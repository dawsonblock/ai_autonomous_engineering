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

    def save_checkpoint(self, namespace: str, thread_id: str, state: dict, parent_thread_id: str | None = None) -> None:
        if hasattr(self.trajectory_memory.store, "save_checkpoint"):
            self.trajectory_memory.store.save_checkpoint(namespace, thread_id, state, parent_thread_id)
            
    def get_checkpoint(self, namespace: str, thread_id: str) -> dict | None:
        if hasattr(self.trajectory_memory.store, "get_checkpoint"):
            return self.trajectory_memory.store.get_checkpoint(namespace, thread_id)
        return None
