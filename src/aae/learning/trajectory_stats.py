from __future__ import annotations

from collections import Counter


class TrajectoryStats:
    def summarize(self, records: list[dict]) -> dict:
        event_counts = Counter(record.get("event_type", "unknown") for record in records)
        workflow_ids = {record.get("workflow_id") for record in records if record.get("workflow_id")}
        success_count = sum(1 for record in records if record.get("event_type") == "task.succeeded")
        failure_count = sum(1 for record in records if record.get("event_type") == "task.failed")
        return {
            "workflow_count": len(workflow_ids),
            "record_count": len(records),
            "event_counts": dict(event_counts),
            "success_rate": success_count / max(success_count + failure_count, 1),
        }
