from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any, Dict


class MemoryStore(ABC):
    @abstractmethod
    def get(self, namespace: str, key: str) -> Any:
        raise NotImplementedError

    @abstractmethod
    def put(self, namespace: str, key: str, value: Any) -> None:
        raise NotImplementedError

    @abstractmethod
    def append(self, namespace: str, key: str, value: Any) -> None:
        raise NotImplementedError

    @abstractmethod
    def snapshot(self, namespace: str) -> Dict[str, Any]:
        raise NotImplementedError
