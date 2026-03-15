from __future__ import annotations

from typing import List

from aae.execution.executor import ActionResult, ActionSpec


class VerificationRule:
    def check(self, action: ActionSpec, result: ActionResult) -> bool:
        return result.success


class TestPassRule(VerificationRule):
    def check(self, action: ActionSpec, result: ActionResult) -> bool:
        if action.action_type != "run_tests":
            return True
        return result.success and "FAILED" not in result.output


class PatchAppliedRule(VerificationRule):
    def check(self, action: ActionSpec, result: ActionResult) -> bool:
        if action.action_type != "apply_patch":
            return True
        return result.success


class Verifier:
    def __init__(self, rules: List[VerificationRule] | None = None) -> None:
        self.rules = rules or [TestPassRule(), PatchAppliedRule()]

    def verify(self, action: ActionSpec, result: ActionResult) -> ActionResult:
        for rule in self.rules:
            if not rule.check(action, result):
                return ActionResult(
                    action_id=action.action_id,
                    success=False,
                    error="verification failed: %s" % type(rule).__name__,
                    artifacts=result.artifacts,
                )
        return result


__all__ = ["PatchAppliedRule", "TestPassRule", "VerificationRule", "Verifier"]
