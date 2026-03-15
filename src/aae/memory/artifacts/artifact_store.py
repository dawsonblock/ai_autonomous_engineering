from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any, Dict, List, Optional
from uuid import uuid4


class Artifact:
    __slots__ = ("artifact_id", "artifact_type", "task_id", "timestamp", "data", "metadata")

    def __init__(
        self,
        artifact_type: str,
        data: Dict[str, Any],
        task_id: str | None = None,
        metadata: Dict[str, Any] | None = None,
    ) -> None:
        self.artifact_id = uuid4().hex
        self.artifact_type = artifact_type
        self.task_id = task_id
        self.timestamp = time.time()
        self.data = data
        self.metadata = metadata or {}

    def to_dict(self) -> Dict[str, Any]:
        return {
            "artifact_id": self.artifact_id,
            "artifact_type": self.artifact_type,
            "task_id": self.task_id,
            "timestamp": self.timestamp,
            "data": self.data,
            "metadata": self.metadata,
        }


class ArtifactStore:
    def __init__(self, base_dir: str = ".artifacts/store") -> None:
        self.base_dir = Path(base_dir)
        self._artifacts: Dict[str, Artifact] = {}
        self._task_index: Dict[str, List[str]] = {}

    def record(self, artifact: Artifact) -> str:
        self._artifacts[artifact.artifact_id] = artifact
        if artifact.task_id:
            self._task_index.setdefault(artifact.task_id, []).append(artifact.artifact_id)
        self._persist(artifact)
        return artifact.artifact_id

    def create(
        self,
        artifact_type: str,
        data: Dict[str, Any],
        task_id: str | None = None,
        metadata: Dict[str, Any] | None = None,
    ) -> Artifact:
        artifact = Artifact(
            artifact_type=artifact_type,
            data=data,
            task_id=task_id,
            metadata=metadata,
        )
        self.record(artifact)
        return artifact

    def get(self, artifact_id: str) -> Artifact | None:
        return self._artifacts.get(artifact_id)

    def by_task(self, task_id: str) -> List[Artifact]:
        ids = self._task_index.get(task_id, [])
        return [self._artifacts[aid] for aid in ids if aid in self._artifacts]

    def by_type(self, artifact_type: str) -> List[Artifact]:
        return [a for a in self._artifacts.values() if a.artifact_type == artifact_type]

    @property
    def count(self) -> int:
        return len(self._artifacts)

    def _persist(self, artifact: Artifact) -> None:
        self.base_dir.mkdir(parents=True, exist_ok=True)
        path = self.base_dir / ("%s.json" % artifact.artifact_id)
        path.write_text(
            json.dumps(artifact.to_dict(), indent=2, sort_keys=True),
            encoding="utf-8",
        )
