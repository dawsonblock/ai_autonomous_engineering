from __future__ import annotations

import json
from pathlib import Path

from aae.contracts.workflow import EventEnvelope
from aae.events.event_bus import EventBus


class EventReplay:
    def __init__(self, event_bus: EventBus) -> None:
        self.event_bus = event_bus

    async def replay(self, path: str | Path) -> None:
        log_path = Path(path)
        with log_path.open("r", encoding="utf-8") as handle:
            for line in handle:
                if not line.strip():
                    continue
                payload = json.loads(line)
                event = EventEnvelope.model_validate(payload)
                await self.event_bus.publish(event, persist=False)
