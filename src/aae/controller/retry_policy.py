from __future__ import annotations

import random
from dataclasses import dataclass

from aae.contracts.results import TaskResult
from aae.contracts.tasks import RetryPolicySpec, TaskSpec


@dataclass(frozen=True)
class RetryDecision:
    should_retry: bool
    delay_s: float = 0.0
    reason: str = ""


class RetryPolicy:
    RETRYABLE_TYPES = {"transport", "timeout", "transient"}

    def __init__(self, rng: random.Random | None = None) -> None:
        self._rng = rng or random.Random()

    def evaluate(self, task: TaskSpec, result: TaskResult) -> RetryDecision:
        if result.error is None:
            return RetryDecision(False, reason="task succeeded")
        if not result.error.transient and result.error.error_type not in self.RETRYABLE_TYPES:
            return RetryDecision(False, reason="error is not retryable")
        if result.attempt >= task.retry_policy.max_attempts:
            return RetryDecision(False, reason="max attempts exhausted")
        delay = self.compute_delay(task.retry_policy, result.attempt)
        return RetryDecision(True, delay_s=delay, reason="transient failure")

    def compute_delay(self, policy: RetryPolicySpec, attempt: int) -> float:
        capped = min(policy.max_delay_s, policy.base_delay_s * (2 ** max(attempt - 1, 0)))
        jitter_window = capped * policy.jitter_ratio
        jitter = self._rng.uniform(-jitter_window, jitter_window)
        return max(0.0, capped + jitter)
