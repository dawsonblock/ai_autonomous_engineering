from __future__ import annotations

import json
from pathlib import Path

from aae.contracts.graph import GraphBuildResult


class GraphMemory:
    def __init__(self, base_dir: str = ".artifacts/memory/graphs") -> None:
        self.base_dir = Path(base_dir)

    def store(self, workflow_id: str, build_result: GraphBuildResult) -> Path:
        self.base_dir.mkdir(parents=True, exist_ok=True)
        path = self.base_dir / ("%s.json" % workflow_id)
        path.write_text(
            json.dumps(build_result.model_dump(mode="json"), indent=2, sort_keys=True),
            encoding="utf-8",
        )
        return path

    def load(self, workflow_id: str) -> dict:
        return json.loads((self.base_dir / ("%s.json" % workflow_id)).read_text(encoding="utf-8"))
