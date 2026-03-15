from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict

from aae.execution.executor import ActionResult, ActionSpec


@dataclass
class SandboxConfig:
    workdir: str = "."
    timeout_seconds: int = 300
    environment: Dict[str, str] = field(default_factory=dict)


class ExecutionSandbox:
    def __init__(self, config: SandboxConfig | None = None) -> None:
        self.config = config or SandboxConfig()
        self._execution_count = 0

    def execute(self, action: ActionSpec) -> ActionResult:
        self._execution_count += 1
        try:
            output = self._run_action(action)
            return ActionResult(
                action_id=action.action_id,
                success=True,
                output=output,
            )
        except Exception as exc:
            return ActionResult(
                action_id=action.action_id,
                success=False,
                error=str(exc),
            )

    def _run_action(self, action: ActionSpec) -> str:
        if action.action_type == "apply_patch":
            return self._apply_patch(action)
        if action.action_type == "run_tests":
            return self._run_tests(action)
        return "executed: %s" % action.action_type

    def _apply_patch(self, action: ActionSpec) -> str:
        patch_content = action.payload.get("patch", "")
        return "patch applied (%d chars)" % len(patch_content)

    def _run_tests(self, action: ActionSpec) -> str:
        test_targets = action.payload.get("tests", [])
        return "tests executed (%d targets)" % len(test_targets)

    @property
    def execution_count(self) -> int:
        return self._execution_count


__all__ = ["ExecutionSandbox", "SandboxConfig"]
