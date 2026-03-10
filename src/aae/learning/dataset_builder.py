from __future__ import annotations


class DatasetBuilder:
    def build(self, trajectories: list[dict]) -> list[dict]:
        return [
            {
                "task_type": record.get("payload", {}).get("task_type") or record.get("event_type", "unknown"),
                "tool": record.get("payload", {}).get("tool") or "graph_query",
                "success": 1 if record.get("event_type") == "task.succeeded" else 0,
                "workflow_id": record.get("workflow_id", ""),
            }
            for record in trajectories
        ]
