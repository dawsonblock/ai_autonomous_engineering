from __future__ import annotations

import json
import math
from pathlib import Path


class VectorMemory:
    def __init__(self, path: str = ".artifacts/memory/vectors.json") -> None:
        self.path = Path(path)

    def put(self, key: str, vector: list[float], metadata: dict | None = None) -> None:
        store = self._load()
        store[key] = {"vector": vector, "metadata": metadata or {}}
        self._save(store)

    def search(self, query: list[float], limit: int = 5) -> list[dict]:
        store = self._load()
        ranked = []
        for key, value in store.items():
            ranked.append(
                {
                    "key": key,
                    "score": _cosine_similarity(query, value["vector"]),
                    "metadata": value.get("metadata", {}),
                }
            )
        return sorted(ranked, key=lambda item: item["score"], reverse=True)[:limit]

    def _load(self) -> dict:
        if not self.path.exists():
            return {}
        return json.loads(self.path.read_text(encoding="utf-8"))

    def _save(self, payload: dict) -> None:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")


def _cosine_similarity(left: list[float], right: list[float]) -> float:
    if not left or not right or len(left) != len(right):
        return 0.0
    dot = sum(a * b for a, b in zip(left, right))
    left_mag = math.sqrt(sum(a * a for a in left))
    right_mag = math.sqrt(sum(b * b for b in right))
    if left_mag == 0 or right_mag == 0:
        return 0.0
    return dot / (left_mag * right_mag)
