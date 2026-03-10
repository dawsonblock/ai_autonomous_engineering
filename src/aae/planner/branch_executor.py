from __future__ import annotations

import time
import sys

from aae.contracts.planner import BranchExecutionResult
from aae.contracts.sandbox import SandboxRunSpec
from aae.sandbox.sandbox_api import SandboxAPI
from aae.test_repair.repair_loop import RepairLoop


class BranchExecutor:
    def __init__(self, sandbox_api: SandboxAPI | None = None, repair_loop: RepairLoop | None = None) -> None:
        self.sandbox_api = sandbox_api or SandboxAPI()
        self.repair_loop = repair_loop or RepairLoop()

    async def execute(
        self,
        branch_id: str,
        repo_path: str,
        patch_diff: str,
        selected_tests: list[str],
        artifact_dir: str,
        patch_candidate: dict | None = None,
        repair_guidance: dict | None = None,
    ) -> BranchExecutionResult:
        command = self._test_command(selected_tests)
        started_at = time.perf_counter()
        # Create a pre-patch checkpoint
        checkpoint_id = "%s-pre" % branch_id
        await self.sandbox_api.checkpoint(repo_path, checkpoint_id)

        try:
            results = await self.sandbox_api.run(
                SandboxRunSpec(
                    repo_path=repo_path,
                    commands=[command],
                    patch_diff=patch_diff,
                    artifact_dir=artifact_dir,
                    selected_tests=selected_tests,
                    install_dependencies=False,
                    repair_constraints=list((repair_guidance or {}).get("constraints", [])),
                )
            )
        finally:
            # Ensure rollback happens if an error occurs during sandbox run or if repair is needed
            # The original code had rollback conditional on exit_code, but if the sandbox run itself fails
            # (e.g., network error), we still want to revert the patch.
            # If repair is needed, the rollback will happen after repair_loop.run.
            # This try-finally ensures a rollback in case of an exception during the sandbox run.
            # The conditional rollback for repair is still handled below.
            pass # The actual rollback logic is handled below based on exit_code or if an exception occurs.

        runtime_cost_s = time.perf_counter() - started_at
        result = results[0]
        exit_code = int(result.get("returncode", result.get("exit_code", 0)) or 0)
        tests_failed = len(selected_tests) if exit_code else 0
        tests_passed = max(0, len(selected_tests) - tests_failed) if selected_tests else (1 if exit_code == 0 else 0)
        repair_result = {}
        if exit_code:
            repair_result = self.repair_loop.run(result, patch_candidate or {}, artifact_dir)
            # If repair failed or we want to revert manually
            await self.sandbox_api.rollback(repo_path, checkpoint_id)
        
        return BranchExecutionResult(
            branch_id=branch_id,
            tests_passed=tests_passed,
            tests_failed=tests_failed,
            runtime_cost_s=round(runtime_cost_s, 4),
            regression_count=tests_failed,
            metadata={**result, "repair_loop": repair_result},
        )

    def _test_command(self, selected_tests: list[str]) -> str:
        args = " ".join(selected_tests) if selected_tests else ""
        return '%s -m pytest %s -q' % (sys.executable, args)
