from __future__ import annotations

from aae.contracts.sandbox import SandboxRunResult, SandboxRunSpec
from aae.sandbox.container_runner import ContainerRunner


class SandboxWorker:
    def __init__(self, runner: ContainerRunner | None = None) -> None:
        self.runner = runner or ContainerRunner()

    async def execute(self, spec: SandboxRunSpec) -> SandboxRunResult:
        return await self.runner.execute(spec)
