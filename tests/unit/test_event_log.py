from pathlib import Path

from aae.core.event_log import EventLog, EventRecord


def test_event_record_fields():
    record = EventRecord(
        event_type="tool_execution",
        task_id="task_27",
        action="patch_generator",
        status="success",
    )
    data = record.to_dict()
    assert data["event"] == "tool_execution"
    assert data["task_id"] == "task_27"
    assert data["action"] == "patch_generator"
    assert data["status"] == "success"
    assert "time" in data
    assert "event_id" in data


def test_event_log_records_events():
    log = EventLog()
    log.create_event(event_type="plan_created", task_id="t1", action="plan")
    log.create_event(event_type="tool_invoked", task_id="t1", action="patch")

    assert log.count == 2
    events = log.get_events(task_id="t1")
    assert len(events) == 2


def test_event_log_filters_by_type():
    log = EventLog()
    log.create_event(event_type="plan_created", task_id="t1")
    log.create_event(event_type="execution_failed", task_id="t1")
    log.create_event(event_type="replan_triggered", task_id="t1")

    failures = log.get_events(event_type="execution_failed")
    assert len(failures) == 1
    assert failures[0]["event"] == "execution_failed"


def test_event_log_persists_to_file(tmp_path: Path):
    log_path = str(tmp_path / "events.jsonl")
    log = EventLog(log_path=log_path)
    log.create_event(event_type="started", task_id="t1")
    log.create_event(event_type="completed", task_id="t1")

    content = Path(log_path).read_text()
    lines = [line for line in content.strip().split("\n") if line]
    assert len(lines) == 2
