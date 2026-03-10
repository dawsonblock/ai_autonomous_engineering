from __future__ import annotations

from aae.contracts.planner import BranchMemoryRecord, PlanBranch


class BranchMemory:
    def __init__(self) -> None:
        self._records: list[BranchMemoryRecord] = []
        self._seen_hashes: set[str] = set()

    def remember(self, branch: PlanBranch, status: str, rejection_reason: str = "") -> None:
        candidate_hash = str(
            (
                tuple(branch.metadata.get("action_sequence", [])),
                tuple(branch.metadata.get("patch_metadata", {}).get("changed_files", [])),
                branch.metadata.get("patch_metadata", {}).get("changed_line_count", 0),
            )
        )
        if status == "explored" and candidate_hash in self._seen_hashes:
            status = "duplicate"
            rejection_reason = rejection_reason or "duplicate branch"
        self._seen_hashes.add(candidate_hash)
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
