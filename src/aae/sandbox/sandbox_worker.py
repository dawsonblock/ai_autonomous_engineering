from __future__ import annotations

from aae.contracts.sandbox import SandboxRunResult, SandboxRunSpec
from aae.sandbox.job_scheduler import JobScheduler


class SandboxWorker:
    def __init__(self, scheduler: JobScheduler | None = None) -> None:
        self.scheduler = scheduler or JobScheduler()

    async def execute(self, spec: SandboxRunSpec) -> SandboxRunResult:
        return await self.scheduler.schedule(spec)
