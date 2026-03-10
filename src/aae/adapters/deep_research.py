from __future__ import annotations

from typing import Any, Dict

from aae.adapters.agentfield_client import AgentFieldClient
from aae.adapters.base import AgentAdapter, now_utc
from aae.contracts.tasks import TaskSpec


class DeepResearchAdapter(AgentAdapter):
    name = "deep_research"
    supported_task_types = ["research"]

    def __init__(
        self,
        client: AgentFieldClient,
        target: str = "meta_deep_research.execute_deep_research",
    ) -> None:
        self.client = client
        self.target = target

    async def execute(self, task: TaskSpec, memory_snapshot: Dict[str, Any]):
        started_at = now_utc()
        attempt = _attempt_from_memory(memory_snapshot, task.task_id) + 1
        try:
            raw = await self.client.execute(self.target, task.payload, task.timeout_s)
            normalized = normalize_research_output(raw)
            return self._success(task, attempt, raw, normalized, started_at)
        except Exception as exc:
            return self._failure(task, attempt, exc, started_at)


def normalize_research_output(raw: Dict[str, Any]) -> Dict[str, Any]:
    package = raw.get("research_package", {})
    metadata = raw.get("metadata") or package.get("metadata", {})
    entities = package.get("entities", [])
    relationships = package.get("relationships", [])
    summary = {
        "entity_count": len(entities) if isinstance(entities, list) else 0,
        "relationship_count": len(relationships) if isinstance(relationships, list) else 0,
        "quality_score": metadata.get("final_quality_score"),
    }
    context = {
        "research": {
            "summary": summary,
            "metadata": metadata,
            "document": package.get("document", {}),
        }
    }
    return {
        "summary": summary,
        "context_for_downstream": context,
        "research_package": package,
        "metadata": metadata,
        "events": [
            {
                "event_type": "research.completed",
                "source": "deep_research_adapter",
                "payload": summary,
            }
        ],
    }


def _attempt_from_memory(memory_snapshot: Dict[str, Any], task_id: str) -> int:
    task_results = memory_snapshot.get("task_results", {})
    return int(task_results.get(task_id, {}).get("attempt", 0))
