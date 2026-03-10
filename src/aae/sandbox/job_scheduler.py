from __future__ import annotations

from aae.contracts.sandbox import SandboxRunResult, SandboxRunSpec
from aae.sandbox.container_manager import ContainerManager


class JobScheduler:
    def __init__(self, container_manager: ContainerManager | None = None) -> None:
        self.container_manager = container_manager or ContainerManager()

    async def schedule(self, spec: SandboxRunSpec) -> SandboxRunResult:
        return await self.container_manager.run(spec)
