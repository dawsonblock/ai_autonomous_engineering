from pathlib import Path

import pytest

from aae.contracts.workflow import EventEnvelope
from aae.events.event_bus import EventBus
from aae.events.event_logger import EventLogger
from aae.events.event_replay import EventReplay


@pytest.mark.anyio
async def test_event_bus_logs_and_replays_events(tmp_path: Path):
    recorded = []
    logger = EventLogger(artifacts_dir=str(tmp_path))
    bus = EventBus(logger=logger)

    async def listener(event):
        recorded.append(event.event_type)

    bus.subscribe("*", listener)
    await bus.publish(
        EventEnvelope(
            event_type="workflow.started",
            workflow_id="wf1",
            source="test",
        )
    )
    await bus.publish(
        EventEnvelope(
            event_type="workflow.completed",
            workflow_id="wf1",
            source="test",
        )
    )

    assert recorded == ["workflow.started", "workflow.completed"]

    replayed = []
    replay_bus = EventBus()

    async def replay_listener(event):
        replayed.append(event.event_type)

    replay_bus.subscribe("*", replay_listener)
    replay = EventReplay(replay_bus)
    await replay.replay(tmp_path / "events" / "wf1.jsonl")

    assert replayed == recorded
