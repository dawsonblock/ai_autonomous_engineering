from __future__ import annotations

from dataclasses import dataclass
from itertools import count


@dataclass
class ContainerLease:
    container_id: str


class ContainerPool:
    def __init__(self) -> None:
        self._counter = count(1)

    def acquire(self) -> ContainerLease:
        return ContainerLease(container_id="local-sandbox-%s" % next(self._counter))

    def release(self, lease: ContainerLease) -> None:
        return None
