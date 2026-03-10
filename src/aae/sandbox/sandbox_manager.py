from __future__ import annotations

from aae.sandbox.container_pool import ContainerPool
from aae.sandbox.job_executor import JobExecutor


class SandboxManager:
    def __init__(
        self,
        container_pool: ContainerPool | None = None,
        job_executor: JobExecutor | None = None,
    ) -> None:
        self.container_pool = container_pool or ContainerPool()
        self.job_executor = job_executor or JobExecutor()

    async def run_job(self, command: str, workdir: str) -> dict:
        lease = self.container_pool.acquire()
        try:
            result = await self.job_executor.run(command, workdir=workdir)
            return {
                "container_id": lease.container_id,
                "command": command,
                "returncode": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
            }
        finally:
            self.container_pool.release(lease)
