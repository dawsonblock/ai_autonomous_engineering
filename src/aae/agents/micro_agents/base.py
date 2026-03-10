from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any, Dict


class BaseMicroAgent(ABC):
    name: str
    domain: str = "swe"

    @abstractmethod
    async def run(self, task: Dict[str, Any], context: Dict[str, Any]) -> Dict[str, Any]:
        raise NotImplementedError
