from __future__ import annotations

from typing import Any, Dict, List


class ReplanDecision:
    __slots__ = ("action", "reason", "retry_step", "alternative_steps")

    def __init__(
        self,
        action: str,
        reason: str = "",
        retry_step: str | None = None,
        alternative_steps: List[str] | None = None,
    ) -> None:
        self.action = action
        self.reason = reason
        self.retry_step = retry_step
        self.alternative_steps = alternative_steps or []


class Replanner:
    def revise(self, result: Dict[str, Any]) -> ReplanDecision:
        if result.get("success", False):
            return ReplanDecision(action="continue", reason="step succeeded")

        error = result.get("error", "")
        failed_step = result.get("step", "")

        if "test" in failed_step.lower() or "test" in error.lower():
            return ReplanDecision(
                action="retry_with_fix",
                reason="test failure detected",
                retry_step=failed_step,
                alternative_steps=["analyze_failure", "generate_alternative_patch", "run_tests"],
            )

        if "patch" in failed_step.lower() or "apply" in error.lower():
            return ReplanDecision(
                action="generate_alternative",
                reason="patch application failed",
                alternative_steps=["generate_alternative_patch", "apply_patch", "run_tests"],
            )

        if "timeout" in error.lower():
            return ReplanDecision(
                action="retry",
                reason="timeout — retrying step",
                retry_step=failed_step,
            )

        return ReplanDecision(
            action="escalate",
            reason="unrecognized failure: %s" % error,
            alternative_steps=["analyze_failure", "propose_patch", "apply_patch", "run_tests"],
        )

    def should_retry(self, result: Dict[str, Any], attempt: int, max_attempts: int = 3) -> bool:
        if result.get("success", False):
            return False
        return attempt < max_attempts
