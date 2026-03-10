from __future__ import annotations

from aae.sandbox.sandbox_manager import SandboxManager


class SandboxAPI:
    def __init__(self, sandbox_manager: SandboxManager | None = None) -> None:
        self.sandbox_manager = sandbox_manager or SandboxManager()

    async def run_tests(self, repo_path: str, commands: list[str]) -> list[dict]:
        return [await self.sandbox_manager.run_job(command, workdir=repo_path) for command in commands]
