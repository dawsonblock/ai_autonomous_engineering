from __future__ import annotations

from aae.contracts.planner import BranchMemoryRecord, PlanBranch


class BranchMemory:
    def __init__(self) -> None:
        self._records: list[BranchMemoryRecord] = []

    def remember(self, branch: PlanBranch, status: str, rejection_reason: str = "") -> None:
        self._records.append(
            BranchMemoryRecord(
                branch_id=branch.branch_id,
                status=status,
                score=branch.score,
                rejection_reason=rejection_reason,
                metadata=branch.metadata,
            )
        )

    def records(self) -> list[BranchMemoryRecord]:
        return list(self._records)
