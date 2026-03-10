from __future__ import annotations

from typing import Dict, Iterable

from aae.agents.micro_agents.base import BaseMicroAgent


class MicroAgentRegistry:
    def __init__(self) -> None:
        self._agents: Dict[str, BaseMicroAgent] = {}

    def register(self, agent: BaseMicroAgent) -> None:
        self._agents[agent.name] = agent

    def get(self, name: str) -> BaseMicroAgent:
        return self._agents[name]

    def list(self) -> Iterable[str]:
        return sorted(self._agents)

    def list_by_domain(self, domain: str) -> list[BaseMicroAgent]:
        return [agent for agent in self._agents.values() if agent.domain == domain]
