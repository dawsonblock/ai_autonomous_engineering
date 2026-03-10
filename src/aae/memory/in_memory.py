from __future__ import annotations

from copy import deepcopy
from threading import RLock
from typing import Any, Dict

from aae.memory.base import MemoryStore


class InMemoryMemoryStore(MemoryStore):
    def __init__(self) -> None:
        self._store: Dict[str, Dict[str, Any]] = {}
        self._lock = RLock()

    def get(self, namespace: str, key: str) -> Any:
        with self._lock:
            return deepcopy(self._store.get(namespace, {}).get(key))

    def put(self, namespace: str, key: str, value: Any) -> None:
        with self._lock:
            self._store.setdefault(namespace, {})[key] = deepcopy(value)

    def append(self, namespace: str, key: str, value: Any) -> None:
        with self._lock:
            bucket = self._store.setdefault(namespace, {})
            bucket.setdefault(key, [])
            bucket[key].append(deepcopy(value))

    def snapshot(self, namespace: str) -> Dict[str, Any]:
        with self._lock:
            return deepcopy(self._store.get(namespace, {}))
