from __future__ import annotations

from aae.contracts.sandbox import SandboxRunSpec
from aae.sandbox.container_pool import ContainerPool
from aae.sandbox.image_builder import ImageBuilder
from aae.sandbox.job_executor import JobExecutor
from aae.sandbox.sandbox_worker import SandboxWorker


class SandboxManager:
    def __init__(
        self,
        container_pool: ContainerPool | None = None,
        job_executor: JobExecutor | None = None,
        image_builder: ImageBuilder | None = None,
        sandbox_worker: SandboxWorker | None = None,
    ) -> None:
        self.container_pool = container_pool or ContainerPool()
        self.job_executor = job_executor or JobExecutor()
        self.image_builder = image_builder or ImageBuilder()
        self.sandbox_worker = sandbox_worker or SandboxWorker()

    async def run_job(self, command: str, workdir: str) -> dict:
        lease = self.container_pool.acquire()
        try:
            spec = SandboxRunSpec(
                repo_path=workdir,
                commands=[command],
                image=self.image_builder.choose_image(workdir),
            )
            result = await self.sandbox_worker.execute(spec)
            if result.transport == "local-fallback":
                local_result = await self.job_executor.run(command, workdir=workdir)
                return {
                    "container_id": lease.container_id,
                    "command": command,
                    "returncode": local_result.returncode,
                    "stdout": local_result.stdout,
                    "stderr": local_result.stderr,
                    "transport": "local-fallback",
                    "patch_apply_status": result.patch_apply_status,
                }
            return {
                "container_id": lease.container_id,
                "command": command,
                "returncode": result.exit_code,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "transport": result.transport,
                "patch_apply_status": result.patch_apply_status,
            }
        finally:
            self.container_pool.release(lease)
