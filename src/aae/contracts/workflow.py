from __future__ import annotations

from datetime import datetime, timezone
from typing import Any, Dict, List, Optional
from uuid import uuid4

from pydantic import BaseModel, Field

from aae.contracts.tasks import TaskSpec


def event_timestamp() -> datetime:
    return datetime.now(timezone.utc)


class WorkflowSpec(BaseModel):
    workflow_id: str
    workflow_type: str
    tasks: List[TaskSpec]
    metadata: Dict[str, Any] = Field(default_factory=dict)


class EventEnvelope(BaseModel):
    event_id: str = Field(default_factory=lambda: str(uuid4()))
    event_type: str
    workflow_id: str
    task_id: Optional[str] = None
    timestamp: datetime = Field(default_factory=event_timestamp)
    source: str
    payload: Dict[str, Any] = Field(default_factory=dict)
