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

    def parse_artifact_dirs(self, paths: list[str | Path]) -> list[dict]:
        records = []
        for path in paths:
            root = Path(path)
            for file_path in root.rglob("benchmark_report.json"):
                try:
                    report = json.loads(file_path.read_text(encoding="utf-8"))
                except json.JSONDecodeError:
                    continue
                for record in report.get("records", []):
                    records.append(
                        {
                            "event_type": "benchmark.case_succeeded" if record.get("fixed") else "benchmark.case_failed",
                            "workflow_id": report.get("run_id", ""),
                            "payload": record,
                        }
                    )
            for file_path in root.rglob("*.jsonl"):
                records.extend(self.parse_jsonl(file_path))
        return records
