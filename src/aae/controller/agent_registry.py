from __future__ import annotations

from typing import Dict, Iterable

from aae.adapters.base import AgentAdapter


class AgentRegistry:
    def __init__(self) -> None:
        self._adapters: Dict[str, AgentAdapter] = {}

    def register(self, adapter: AgentAdapter) -> None:
        if adapter.name in self._adapters:
            raise ValueError("adapter '%s' is already registered" % adapter.name)
        self._adapters[adapter.name] = adapter

    def get(self, name: str) -> AgentAdapter:
        try:
            return self._adapters[name]
        except KeyError as exc:
            raise KeyError("adapter '%s' is not registered" % name) from exc

    def list(self) -> Iterable[str]:
        return sorted(self._adapters)
