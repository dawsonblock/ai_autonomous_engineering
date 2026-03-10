from __future__ import annotations

from datetime import datetime, timezone
from enum import Enum
from typing import Any, Dict, Optional

from pydantic import BaseModel, Field


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


class TaskResultStatus(str, Enum):
    SUCCEEDED = "succeeded"
    FAILED = "failed"


class TaskError(BaseModel):
    message: str
    error_type: str = "unknown"
    transient: bool = False
    details: Dict[str, Any] = Field(default_factory=dict)


class TaskResult(BaseModel):
    task_id: str
    status: TaskResultStatus
    raw_output: Dict[str, Any] = Field(default_factory=dict)
    normalized_output: Dict[str, Any] = Field(default_factory=dict)
    error: Optional[TaskError] = None
    attempt: int = 1
    started_at: datetime = Field(default_factory=utc_now)
    finished_at: datetime = Field(default_factory=utc_now)
