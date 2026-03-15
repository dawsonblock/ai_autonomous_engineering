from aae.core.event_log import EventLog
from aae.execution.executor import ActionResult, ActionSpec, ExecutionPolicy, Executor
from aae.execution.sandbox import ExecutionSandbox
from aae.execution.verifier import TestPassRule, Verifier


def test_executor_runs_action():
    executor = Executor()
    action = ActionSpec(action_id="a1", action_type="analyze_code")
    result = executor.run(action)

    assert result.success
    assert result.action_id == "a1"


# ── policy rejection ──────────────────────────────────────────────


def test_executor_rejects_empty_action_id():
    executor = Executor()
    action = ActionSpec(action_id="", action_type="analyze_code")
    result = executor.run(action)

    assert not result.success
    assert "rejected" in result.error


def test_executor_rejects_empty_action_type():
    executor = Executor()
    action = ActionSpec(action_id="a1", action_type="")
    result = executor.run(action)

    assert not result.success
    assert "rejected" in result.error


def test_policy_rejection_logs_rejected_event():
    event_log = EventLog()
    executor = Executor(event_log=event_log)
    action = ActionSpec(action_id="", action_type="run_tests")
    executor.run(action)

    events = event_log.get_events(event_type="action_rejected")
    assert len(events) == 1
    assert events[0]["payload"]["reason"] == "policy_validation_failed"


# ── sandbox execution ─────────────────────────────────────────────


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


def test_executor_sandbox_failure_propagates():
    class FailingSandbox:
        def execute(self, action):
            return ActionResult(
                action_id=action.action_id,
                success=False,
                error="sandbox error: container crashed",
            )

    executor = Executor(sandbox=FailingSandbox())
    action = ActionSpec(action_id="a1", action_type="run_tests")
    result = executor.run(action)

    assert not result.success
    assert "container crashed" in result.error


def test_executor_sandbox_exception_is_caught():
    class ExplodingSandbox:
        def execute(self, action):
            raise RuntimeError("unexpected sandbox failure")

    event_log = EventLog()
    executor = Executor(sandbox=ExplodingSandbox(), event_log=event_log)
    action = ActionSpec(action_id="a1", action_type="run_tests")
    result = executor.run(action)

    assert not result.success
    assert "unexpected sandbox failure" in result.error

    failed_events = event_log.get_events(event_type="action_failed")
    assert len(failed_events) == 1
    assert failed_events[0]["payload"]["error"] == "unexpected sandbox failure"


# ── verifier failure propagation ──────────────────────────────────


def test_executor_with_verifier():
    verifier = Verifier()
    executor = Executor(verifier=verifier)
    action = ActionSpec(action_id="a1", action_type="run_tests")
    result = executor.run(action)

    assert result.success


def test_verifier_failure_propagates_through_executor():
    verifier = Verifier(rules=[TestPassRule()])
    event_log = EventLog()
    executor = Executor(verifier=verifier, event_log=event_log)

    action = ActionSpec(action_id="a1", action_type="run_tests")
    # _execute_local returns success, but output doesn't contain "FAILED"
    # so TestPassRule passes.  Override the sandbox to inject a FAILED output.

    class TestFailSandbox:
        def execute(self, action):
            return ActionResult(
                action_id=action.action_id,
                success=True,
                output="FAILED test_something",
            )

    executor.sandbox = TestFailSandbox()
    result = executor.run(action)

    assert not result.success
    assert "TestPassRule" in result.error
    assert "FAILED test_something" in result.output  # output preserved

    verification_events = event_log.get_events(event_type="verification_failed")
    assert len(verification_events) == 1


def test_verifier_preserves_original_error_on_failure():
    verifier = Verifier(rules=[TestPassRule()])

    class PartialFailSandbox:
        def execute(self, action):
            return ActionResult(
                action_id=action.action_id,
                success=True,
                output="FAILED test_foo",
                error="some upstream warning",
                artifacts={"log": "trace.txt"},
            )

    executor = Executor(verifier=verifier, sandbox=PartialFailSandbox())
    result = executor.run(ActionSpec(action_id="a1", action_type="run_tests"))

    assert not result.success
    assert "some upstream warning" in result.error
    assert "TestPassRule" in result.error
    assert result.artifacts == {"log": "trace.txt"}


# ── event log side effects ────────────────────────────────────────


def test_executor_logs_events():
    event_log = EventLog()
    executor = Executor(event_log=event_log)
    action = ActionSpec(action_id="a1", action_type="run_tests")
    executor.run(action)

    assert event_log.count >= 2  # started + completed


def test_successful_run_emits_started_and_completed():
    event_log = EventLog()
    executor = Executor(event_log=event_log)
    executor.run(ActionSpec(action_id="a1", action_type="analyze_code"))

    started = event_log.get_events(event_type="action_started")
    completed = event_log.get_events(event_type="action_completed")
    assert len(started) == 1
    assert len(completed) == 1
    assert started[0]["task_id"] == "a1"
    assert completed[0]["action"] == "analyze_code"


def test_rejection_emits_started_and_rejected():
    event_log = EventLog()
    executor = Executor(event_log=event_log)
    executor.run(ActionSpec(action_id="", action_type="run_tests"))

    started = event_log.get_events(event_type="action_started")
    rejected = event_log.get_events(event_type="action_rejected")
    assert len(started) == 1
    assert len(rejected) == 1


def test_verification_failure_emits_verification_event():
    event_log = EventLog()
    verifier = Verifier(rules=[TestPassRule()])

    class FailOutput:
        def execute(self, action):
            return ActionResult(action_id=action.action_id, success=True, output="FAILED x")

    executor = Executor(sandbox=FailOutput(), verifier=verifier, event_log=event_log)
    executor.run(ActionSpec(action_id="a1", action_type="run_tests"))

    started = event_log.get_events(event_type="action_started")
    verification_failed = event_log.get_events(event_type="verification_failed")
    assert len(started) == 1
    assert len(verification_failed) == 1
    assert event_log.get_events(event_type="action_completed") == []
