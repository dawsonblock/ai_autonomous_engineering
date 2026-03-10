from random import Random

from aae.contracts.results import TaskError, TaskResult, TaskResultStatus
from aae.contracts.tasks import TaskSpec
from aae.controller.retry_policy import RetryPolicy


def test_retry_policy_retries_transient_failures():
    policy = RetryPolicy(rng=Random(0))
    task = TaskSpec(task_id="t1", task_type="research", agent_name="deep_research")
    result = TaskResult(
        task_id="t1",
        status=TaskResultStatus.FAILED,
        attempt=1,
        error=TaskError(message="timeout", error_type="timeout", transient=True),
    )

    decision = policy.evaluate(task, result)

    assert decision.should_retry is True
    assert decision.delay_s > 0


def test_retry_policy_stops_on_non_retryable_errors():
    policy = RetryPolicy(rng=Random(0))
    task = TaskSpec(task_id="t1", task_type="research", agent_name="deep_research")
    result = TaskResult(
        task_id="t1",
        status=TaskResultStatus.FAILED,
        attempt=1,
        error=TaskError(message="bad request", error_type="validation", transient=False),
    )

    decision = policy.evaluate(task, result)

    assert decision.should_retry is False
