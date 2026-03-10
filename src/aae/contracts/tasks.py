from __future__ import annotations

from enum import Enum
from typing import Any, Dict, List

from pydantic import BaseModel, Field, field_validator


class TaskState(str, Enum):
    PENDING = "pending"
    READY = "ready"
    RUNNING = "running"
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    RETRY_WAITING = "retry_waiting"
    BLOCKED = "blocked"
    CANCELLED = "cancelled"


class RetryPolicySpec(BaseModel):
    max_attempts: int = 3
    base_delay_s: float = 2.0
    max_delay_s: float = 30.0
    jitter_ratio: float = 0.2

    @field_validator("max_attempts")
    @classmethod
    def _validate_attempts(cls, value: int) -> int:
        if value < 1:
            raise ValueError("max_attempts must be at least 1")
        return value


class TaskSpec(BaseModel):
    task_id: str
    task_type: str
    agent_name: str
    payload: Dict[str, Any] = Field(default_factory=dict)
    depends_on: List[str] = Field(default_factory=list)
    priority: int = 0
    timeout_s: float = 300.0
    retry_policy: RetryPolicySpec = Field(default_factory=RetryPolicySpec)
    soft_dependencies: List[str] = Field(default_factory=list)

    @field_validator("soft_dependencies")
    @classmethod
    def _validate_soft_deps(cls, value: List[str], info) -> List[str]:
        depends_on = info.data.get("depends_on", [])
        unknown = sorted(set(value) - set(depends_on))
        if unknown:
            raise ValueError(
                "soft_dependencies must be a subset of depends_on; "
                "unknown soft deps: %s" % ", ".join(unknown)
            )
        return value
