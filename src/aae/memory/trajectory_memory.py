from __future__ import annotations

import json
from pathlib import Path


class TrajectoryMemory:
    def __init__(self, base_dir: str = ".artifacts/memory/trajectories") -> None:
        self.base_dir = Path(base_dir)

    def append(self, namespace: str, record: dict) -> Path:
        self.base_dir.mkdir(parents=True, exist_ok=True)
        path = self.base_dir / ("%s.jsonl" % namespace)
        with path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(record, sort_keys=True))
            handle.write("\n")
        return path

    def read(self, namespace: str) -> list[dict]:
        path = self.base_dir / ("%s.jsonl" % namespace)
        if not path.exists():
            return []
        return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
