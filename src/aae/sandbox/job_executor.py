from __future__ import annotations

import asyncio
from dataclasses import dataclass


@dataclass
class SandboxJobResult:
    command: str
    returncode: int
    stdout: str
    stderr: str


class JobExecutor:
    async def run(self, command: str, workdir: str) -> SandboxJobResult:
        process = await asyncio.create_subprocess_shell(
            command,
            cwd=workdir,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        stdout, stderr = await process.communicate()
        return SandboxJobResult(
            command=command,
            returncode=process.returncode,
            stdout=stdout.decode("utf-8", "ignore"),
            stderr=stderr.decode("utf-8", "ignore"),
        )
