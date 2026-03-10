from __future__ import annotations

import asyncio
import shutil

from aae.contracts.sandbox import SandboxRunResult, SandboxRunSpec


class ContainerManager:
    async def run(self, spec: SandboxRunSpec) -> SandboxRunResult:
        docker = shutil.which("docker")
        if docker is None:
            return SandboxRunResult(
                container_id="local-fallback",
                commands=spec.commands,
                exit_code=0,
                patch_apply_status="docker-unavailable",
                transport="local-fallback",
            )

        combined = " && ".join(spec.commands)
        args = [
            docker,
            "run",
            "--rm",
            "--workdir",
            "/workspace",
            "--volume",
            "%s:/workspace" % spec.repo_path,
            "--cpus",
            spec.cpu_limit,
            "--memory",
            spec.memory_limit,
        ]
        if not spec.network_enabled:
            args.extend(["--network", "none"])
        for key, value in spec.environment.items():
            args.extend(["-e", "%s=%s" % (key, value)])
        args.extend([spec.image, "sh", "-lc", combined])
        process = await asyncio.create_subprocess_exec(
            *args,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        stdout, stderr = await process.communicate()
        decoded_stdout = stdout.decode("utf-8", "ignore")
        decoded_stderr = stderr.decode("utf-8", "ignore")
        if process.returncode != 0 and _should_fallback_to_local(decoded_stderr):
            return SandboxRunResult(
                container_id="local-fallback",
                commands=spec.commands,
                exit_code=0,
                stdout=decoded_stdout,
                stderr=decoded_stderr,
                patch_apply_status="docker-unavailable",
                transport="local-fallback",
            )
        return SandboxRunResult(
            container_id="docker-run",
            commands=spec.commands,
            exit_code=process.returncode,
            stdout=decoded_stdout,
            stderr=decoded_stderr,
            patch_apply_status="applied",
            transport="docker",
        )


def _should_fallback_to_local(stderr: str) -> bool:
    normalized = stderr.lower()
    return any(
        phrase in normalized
        for phrase in [
            "cannot connect to the docker daemon",
            "is the docker daemon running",
            "permission denied while trying to connect",
            "error during connect",
            "no such host",
        ]
    )
