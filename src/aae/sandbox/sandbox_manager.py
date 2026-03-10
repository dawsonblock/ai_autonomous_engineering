from __future__ import annotations

import tempfile
import uuid
from pathlib import Path

from aae.contracts.sandbox import SandboxRunSpec
from aae.sandbox.artifact_collector import ArtifactCollector
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
        artifact_collector: ArtifactCollector | None = None,
    ) -> None:
        self.container_pool = container_pool or ContainerPool()
        self.job_executor = job_executor or JobExecutor()
        self.image_builder = image_builder or ImageBuilder()
        self.sandbox_worker = sandbox_worker or SandboxWorker()
        self.artifact_collector = artifact_collector or ArtifactCollector()

    async def run_job(self, command: str, workdir: str) -> dict:
        spec = SandboxRunSpec(
            repo_path=workdir,
            commands=[command],
            image=self.image_builder.choose_image(workdir),
            command_id="cmd-%s" % uuid.uuid4().hex[:8],
        )
        result = await self.execute_spec(spec)
        return {
            "container_id": result.container_id,
            "command": command,
            "returncode": result.exit_code,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "transport": result.transport,
            "patch_apply_status": result.patch_apply_status,
            "artifact_paths": result.artifact_paths,
            "trace_paths": result.trace_paths,
            "coverage_path": result.coverage_path,
            "applied_workspace": result.applied_workspace,
            "editable_workspace": result.editable_workspace,
            "rollback_status": result.rollback_status,
            "counterexample_paths": result.counterexample_paths,
            "patch_apply_details": result.patch_apply_details,
            "command_id": spec.command_id,
            "selected_tests": spec.selected_tests,
        }

    async def execute_spec(self, spec: SandboxRunSpec):
        lease = self.container_pool.acquire()
        try:
            artifact_dir = spec.artifact_dir or str(Path(tempfile.mkdtemp(prefix="aae-sandbox-")) / lease.container_id)
            spec = spec.model_copy(update={"artifact_dir": artifact_dir, "image": spec.image or self.image_builder.choose_image(spec.repo_path), "command_id": spec.command_id or "cmd-%s" % uuid.uuid4().hex[:8]})
            workspace, trace_path, patch_metadata = self.artifact_collector.prepare_workspace(spec, lease.container_id)
            umbrella_src = str(Path(__file__).resolve().parents[2])
            existing_pythonpath = spec.environment.get("PYTHONPATH", "")
            env = {
                **spec.environment,
                "AAE_TRACE_OUTPUT": trace_path if spec.trace_enabled else "",
                "AAE_TRACE_FILTER_ROOT": workspace if spec.trace_enabled else "",
                "AAE_TRACE_COMMAND_ID": spec.command_id,
                "PYTHONPATH": "%s%s%s" % (umbrella_src, ":" if existing_pythonpath else "", existing_pythonpath),
            }
            prepared_spec = spec.model_copy(update={"repo_path": workspace, "environment": env})
            result = await self.sandbox_worker.execute(prepared_spec)
            if result.transport == "local-fallback":
                command = spec.commands[0] if spec.commands else ""
                local_result = await self.job_executor.run(command, workdir=workspace, environment=env)
                return self.artifact_collector.collect(
                    result.model_copy(
                        update={
                            "container_id": lease.container_id,
                            "exit_code": local_result.returncode,
                            "stdout": local_result.stdout,
                            "stderr": local_result.stderr,
                            "applied_workspace": workspace,
                            "trace_paths": [trace_path] if Path(trace_path).exists() else [],
                            "artifact_paths": [trace_path] if Path(trace_path).exists() else [],
                            "dependency_install_status": "skipped",
                            "patch_apply_status": patch_metadata["patch_apply_status"] or ("applied" if not spec.patch_diff or local_result.returncode >= 0 else result.patch_apply_status),
                            "rollback_status": patch_metadata["rollback_status"],
                            "editable_workspace": patch_metadata["editable_workspace"],
                            "counterexample_paths": patch_metadata["counterexample_paths"],
                            "patch_apply_details": patch_metadata["patch_apply_details"],
                        }
                    )
                )
            return self.artifact_collector.collect(
                result.model_copy(
                    update={
                        "container_id": lease.container_id,
                        "applied_workspace": workspace,
                        "trace_paths": [trace_path] if Path(trace_path).exists() else [],
                        "artifact_paths": [trace_path] if Path(trace_path).exists() else [],
                        "dependency_install_status": "skipped",
                        "patch_apply_status": patch_metadata["patch_apply_status"] or result.patch_apply_status,
                        "rollback_status": patch_metadata["rollback_status"],
                        "editable_workspace": patch_metadata["editable_workspace"],
                        "counterexample_paths": patch_metadata["counterexample_paths"],
                        "patch_apply_details": patch_metadata["patch_apply_details"],
                    }
                )
            )
        finally:
            self.container_pool.release(lease)
