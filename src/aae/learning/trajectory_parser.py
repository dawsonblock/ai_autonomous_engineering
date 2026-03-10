from __future__ import annotations

import json
from pathlib import Path


class TrajectoryParser:
    def parse_jsonl(self, path: str | Path) -> list[dict]:
        records = []
        for line in Path(path).read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            records.append(json.loads(line))
        return records

    def parse_many(self, paths: list[str | Path]) -> list[dict]:
        records = []
        for path in paths:
            records.extend(self.parse_jsonl(path))
        return records
