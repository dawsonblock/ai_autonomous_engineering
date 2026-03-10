from __future__ import annotations

from abc import ABC, abstractmethod
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional

import httpx
from pydantic import BaseModel

from aae.contracts.results import TaskError, TaskResult, TaskResultStatus
from aae.contracts.tasks import TaskSpec


def now_utc() -> datetime:
    return datetime.now(timezone.utc)


class AgentAdapter(ABC):
    name: str
    supported_task_types: List[str]

    @abstractmethod
    async def execute(self, task: TaskSpec, memory_snapshot: Dict[str, Any]) -> TaskResult:
        raise NotImplementedError

    def _success(
        self,
        task: TaskSpec,
        attempt: int,
        raw_output: Dict[str, Any],
        normalized_output: Dict[str, Any],
        started_at: datetime,
    ) -> TaskResult:
        return TaskResult(
            task_id=task.task_id,
            status=TaskResultStatus.SUCCEEDED,
            raw_output=raw_output,
            normalized_output=normalized_output,
            attempt=attempt,
            started_at=started_at,
            finished_at=now_utc(),
        )

    def _failure(
        self,
        task: TaskSpec,
        attempt: int,
        error: Exception,
        started_at: datetime,
        error_type: Optional[str] = None,
        transient: Optional[bool] = None,
    ) -> TaskResult:
        if transient is None:
            transient = isinstance(error, (httpx.TransportError, httpx.TimeoutException, TransientAdapterError))
        kind = error_type or infer_error_type(error)
        return TaskResult(
            task_id=task.task_id,
            status=TaskResultStatus.FAILED,
            error=TaskError(
                message=str(error),
                error_type=kind,
                transient=transient,
            ),
            attempt=attempt,
            started_at=started_at,
            finished_at=now_utc(),
        )


class TransientAdapterError(RuntimeError):
    """Adapter-visible transient failure."""


def infer_error_type(error: Exception) -> str:
    if isinstance(error, httpx.TimeoutException):
        return "timeout"
    if isinstance(error, httpx.TransportError):
        return "transport"
    if isinstance(error, TransientAdapterError):
        return "transient"
    return "adapter"
