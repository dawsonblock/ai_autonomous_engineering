from __future__ import annotations

import asyncio
import os
import shlex
import shutil
from dataclasses import dataclass


@dataclass
class SandboxJobResult:
    command: str
    returncode: int
    stdout: str
    stderr: str


class JobExecutor:
    async def run(self, command: str, workdir: str, environment: dict[str, str] | None = None) -> SandboxJobResult:
        argv = _argv_for_command(command)
        env = os.environ.copy()
        if environment:
            env.update(environment)
        process = await asyncio.create_subprocess_exec(
            *argv,
            cwd=workdir,
            env=env,
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


def _argv_for_command(command: str) -> list[str]:
    argv = shlex.split(command)
    if not argv:
        raise ValueError("command must not be empty")
    if _is_executable(argv[0]):
        return argv
    reconstructed = _reconstruct_executable(argv)
    return reconstructed or argv


def _reconstruct_executable(argv: list[str]) -> list[str] | None:
    if len(argv) < 2:
        return None
    for index in range(2, len(argv) + 1):
        candidate = " ".join(argv[:index])
        if _is_executable(candidate):
            return [candidate, *argv[index:]]
    return None


def _is_executable(value: str) -> bool:
    if not value:
        return False
    if os.path.isfile(value) and os.access(value, os.X_OK):
        return True
    return shutil.which(value) is not None
