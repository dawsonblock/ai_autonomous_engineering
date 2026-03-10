from __future__ import annotations

import asyncio
from pathlib import Path
from typing import Any, Dict, List

from aae.planner.branch_executor import BranchExecutor


class ExperimentRunner:
    def __init__(
        self,
        branch_executor: BranchExecutor | None = None,
        max_branches: int = 8,
        parallel_workers: int = 4,
    ) -> None:
        self.branch_executor = branch_executor or BranchExecutor()
        self.max_branches = max_branches
        self.parallel_workers = parallel_workers

    async def run(self, repo_path: str, branches: List[Dict[str, Any]], artifacts_dir: str) -> List[Dict[str, Any]]:
        semaphore = asyncio.Semaphore(self.parallel_workers)

        async def _run(branch: Dict[str, Any]) -> Dict[str, Any]:
            async with semaphore:
                execution = await self.branch_executor.execute(
                    branch_id=branch["branch_id"],
                    repo_path=repo_path,
                    patch_diff=branch["patch_candidate"].get("diff", ""),
                    selected_tests=list(branch.get("selected_tests", [])),
                    artifact_dir=str(Path(artifacts_dir) / branch["branch_id"]),
                    patch_candidate=branch["patch_candidate"],
                    repair_guidance=branch["patch_candidate"].get("repair_guidance", {}),
                )
                return {
                    "branch_id": branch["branch_id"],
                    "patch_candidate": branch["patch_candidate"],
                    "selected_tests": branch.get("selected_tests", []),
                    "execution": execution.model_dump(mode="json"),
                    "metadata": branch.get("metadata", {}),
                }

        tasks = [_run(branch) for branch in branches[: self.max_branches] if branch.get("patch_candidate", {}).get("diff")]
        return await asyncio.gather(*tasks) if tasks else []
