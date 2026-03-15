from aae.core.event_log import EventLog
from aae.execution.executor import ActionResult, ActionSpec, Executor
from aae.execution.sandbox import ExecutionSandbox
from aae.execution.verifier import Verifier


def test_executor_runs_action():
    executor = Executor()
    action = ActionSpec(action_id="a1", action_type="analyze_code")
    result = executor.run(action)

    assert result.success
    assert result.action_id == "a1"


def test_executor_rejects_invalid_action():
    executor = Executor()
    action = ActionSpec(action_id="", action_type="analyze_code")
    result = executor.run(action)

    assert not result.success
    assert "rejected" in result.error


def test_executor_logs_events():
    event_log = EventLog()
    executor = Executor(event_log=event_log)
    action = ActionSpec(action_id="a1", action_type="run_tests")
    executor.run(action)

    assert event_log.count >= 2  # started + completed


def test_executor_with_sandbox():
    sandbox = ExecutionSandbox()
    executor = Executor(sandbox=sandbox)
    action = ActionSpec(
        action_id="a1",
        action_type="apply_patch",
        payload={"patch": "diff --git a/file.py"},
    )
    result = executor.run(action)

    assert result.success
    assert sandbox.execution_count == 1


def test_executor_with_verifier():
    verifier = Verifier()
    executor = Executor(verifier=verifier)
    action = ActionSpec(action_id="a1", action_type="run_tests")
    result = executor.run(action)

    assert result.success
