from __future__ import annotations

from aae.contracts.sandbox import SandboxRunSpec
from aae.sandbox.sandbox_manager import SandboxManager


class SandboxAPI:
    def __init__(self, sandbox_manager: SandboxManager | None = None) -> None:
        self.sandbox_manager = sandbox_manager or SandboxManager()

    async def run_tests(self, repo_path: str, commands: list[str]) -> list[dict]:
        return [await self.sandbox_manager.run_job(command, workdir=repo_path) for command in commands]

    async def run(self, spec: SandboxRunSpec) -> list[dict]:
        if len(spec.commands) == 1:
            result = await self.sandbox_manager.execute_spec(spec)
            payload = result.model_dump(mode="json")
            payload["returncode"] = payload.get("exit_code", 0)
            return [payload]
        results = []
        for command in spec.commands:
            command_spec = spec.model_copy(update={"commands": [command]})
            result = await self.sandbox_manager.execute_spec(command_spec)
            payload = result.model_dump(mode="json")
            payload["returncode"] = payload.get("exit_code", 0)
            results.append(payload)
        return results

    async def checkpoint(self, repo_path: str, checkpoint_id: str) -> bool:
        return await self.sandbox_manager.checkpoint(repo_path, checkpoint_id)

    async def rollback(self, repo_path: str, checkpoint_id: str) -> bool:
        return await self.sandbox_manager.rollback(repo_path, checkpoint_id)
