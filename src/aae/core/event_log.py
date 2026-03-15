from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any, Dict, List
from uuid import uuid4


class EventRecord:
    __slots__ = ("event_id", "timestamp", "task_id", "event_type", "action", "status", "payload")

    def __init__(
        self,
        event_type: str,
        task_id: str | None = None,
        action: str | None = None,
        status: str | None = None,
        payload: Dict[str, Any] | None = None,
    ) -> None:
        self.event_id = uuid4().hex
        self.timestamp = time.time()
        self.task_id = task_id
        self.event_type = event_type
        self.action = action
        self.status = status
        self.payload = payload or {}

    def to_dict(self) -> Dict[str, Any]:
        return {
            "event_id": self.event_id,
            "time": self.timestamp,
            "task_id": self.task_id,
            "event": self.event_type,
            "action": self.action,
            "status": self.status,
            "payload": self.payload,
        }


class EventLog:
    def __init__(self, log_path: str | None = None) -> None:
        self._records: List[EventRecord] = []
        self._log_path = Path(log_path) if log_path else None
        self._log_dir_created = False

    def record(self, event: EventRecord) -> None:
        self._records.append(event)
        if self._log_path is not None:
            self._persist(event)

    def create_event(
        self,
        event_type: str,
        task_id: str | None = None,
        action: str | None = None,
        status: str | None = None,
        payload: Dict[str, Any] | None = None,
    ) -> EventRecord:
        event = EventRecord(
            event_type=event_type,
            task_id=task_id,
            action=action,
            status=status,
            payload=payload,
        )
        self.record(event)
        return event

    def get_events(self, task_id: str | None = None, event_type: str | None = None) -> List[Dict[str, Any]]:
        filtered = self._records
        if task_id is not None:
            filtered = [record for record in filtered if record.task_id == task_id]
        if event_type is not None:
            filtered = [record for record in filtered if record.event_type == event_type]
        return [record.to_dict() for record in filtered]

    @property
    def count(self) -> int:
        return len(self._records)

    def _persist(self, event: EventRecord) -> None:
        if not self._log_dir_created:
            self._log_path.parent.mkdir(parents=True, exist_ok=True)
            self._log_dir_created = True
        with self._log_path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(event.to_dict(), sort_keys=True))
            handle.write("\n")


__all__ = ["EventLog", "EventRecord"]
