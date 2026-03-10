import pytest

from aae.adapters.base import AgentAdapter
from aae.controller.agent_registry import AgentRegistry
from aae.memory.in_memory import InMemoryMemoryStore


class DummyAdapter(AgentAdapter):
    name = "dummy"
    supported_task_types = ["dummy"]

    async def execute(self, task, memory_snapshot):
        raise NotImplementedError


def test_registry_rejects_duplicate_registration():
    registry = AgentRegistry()
    registry.register(DummyAdapter())

    with pytest.raises(ValueError):
        registry.register(DummyAdapter())


def test_memory_snapshots_are_isolated():
    memory = InMemoryMemoryStore()
    memory.put("workflow/a", "value", {"count": 1})
    snapshot = memory.snapshot("workflow/a")
    snapshot["value"]["count"] = 99

    assert memory.get("workflow/a", "value") == {"count": 1}
    assert memory.snapshot("workflow/b") == {}
