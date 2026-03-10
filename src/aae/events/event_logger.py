from __future__ import annotations

import json
from pathlib import Path

from aae.contracts.workflow import EventEnvelope


class EventLogger:
    def __init__(self, artifacts_dir: str = ".artifacts") -> None:
        self.artifacts_dir = Path(artifacts_dir)

    async def append(self, event: EventEnvelope) -> Path:
        event_dir = self.artifacts_dir / "events"
        event_dir.mkdir(parents=True, exist_ok=True)
        path = event_dir / ("%s.jsonl" % event.workflow_id)
        with path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(event.model_dump(mode="json"), sort_keys=True))
            handle.write("\n")
        return path
