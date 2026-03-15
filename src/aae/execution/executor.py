from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict

from aae.core.event_log import EventLog, EventRecord


@dataclass
class ActionSpec:
    action_id: str
    action_type: str
    command: str | None = None
    payload: Dict[str, Any] = field(default_factory=dict)


@dataclass
class ActionResult:
    action_id: str
    success: bool
    output: str = ""
    error: str = ""
    artifacts: Dict[str, Any] = field(default_factory=dict)


class ExecutionPolicy:
    def validate(self, action: ActionSpec) -> bool:
        if not action.action_id:
            return False
        if not action.action_type:
            return False
        return True


class Executor:
    def __init__(
        self,
        policy: ExecutionPolicy | None = None,
        sandbox: Any | None = None,
        verifier: Any | None = None,
        event_log: EventLog | None = None,
    ) -> None:
        self.policy = policy or ExecutionPolicy()
        self.sandbox = sandbox
        self.verifier = verifier
        self.event_log = event_log or EventLog()

    def run(self, action: ActionSpec) -> ActionResult:
        self.event_log.create_event(
            event_type="action_started",
            task_id=action.action_id,
            action=action.action_type,
            status="started",
        )

        if not self.policy.validate(action):
            self.event_log.create_event(
                event_type="action_rejected",
                task_id=action.action_id,
                action=action.action_type,
                status="rejected",
                payload={"reason": "policy_validation_failed"},
            )
            return ActionResult(
                action_id=action.action_id,
                success=False,
                error="action rejected by execution policy",
            )

        try:
            if self.sandbox is not None:
                result = self.sandbox.execute(action)
            else:
                result = self._execute_local(action)
        except Exception as exc:
            self.event_log.create_event(
                event_type="action_failed",
                task_id=action.action_id,
                action=action.action_type,
                status="error",
                payload={"error": str(exc)},
            )
            return ActionResult(
                action_id=action.action_id,
                success=False,
                error=str(exc),
            )

        if self.verifier is not None:
            verified = self.verifier.verify(action, result)
            if not verified.success:
                self.event_log.create_event(
                    event_type="verification_failed",
                    task_id=action.action_id,
                    action=action.action_type,
                    status="verification_failed",
                )
                return verified

        self.event_log.create_event(
            event_type="action_completed",
            task_id=action.action_id,
            action=action.action_type,
            status="success",
        )
        return result

    def _execute_local(self, action: ActionSpec) -> ActionResult:
        return ActionResult(
            action_id=action.action_id,
            success=True,
            output="executed: %s" % action.action_type,
        )


__all__ = ["ActionResult", "ActionSpec", "ExecutionPolicy", "Executor"]
