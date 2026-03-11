from __future__ import annotations

from pathlib import Path

from aae.memory.trajectory_memory import TrajectoryMemory
from aae.patching.patch_provenance import PatchProvenanceRecorder
from aae.persistence.trajectory_store import PostgresTrajectoryStore


class PersistenceService:
    def __init__(
        self,
        artifacts_dir: str,
        trajectory_memory: TrajectoryMemory | None = None,
        persistent_trajectory_store: PostgresTrajectoryStore | None = None,
        provenance_recorder: PatchProvenanceRecorder | None = None,
    ) -> None:
        self.artifacts_dir = artifacts_dir
        self.trajectory_memory = trajectory_memory or TrajectoryMemory(base_dir=str(Path(artifacts_dir) / "memory" / "trajectories"))
        self.persistent_trajectory_store = persistent_trajectory_store or PostgresTrajectoryStore()
        self.provenance_recorder = provenance_recorder or PatchProvenanceRecorder()

    def save_checkpoint(self, namespace: str, thread_id: str, state: dict, parent_thread_id: str | None = None) -> None:
        self.persistent_trajectory_store.save_checkpoint(
            namespace=namespace,
            thread_id=thread_id,
            state=state,
            parent_thread_id=parent_thread_id,
        )

    def append_trajectory(self, namespace: str, record: dict) -> None:
        self.trajectory_memory.append(namespace, record)
        self.persistent_trajectory_store.append(namespace, record)

    def record_patch_provenance(self, workflow_id: str, task_id: str, localization_result: dict, exploration_results: list[dict]) -> list[dict]:
        records = []
        artifact_path = Path(self.artifacts_dir) / "patch_provenance" / f"{workflow_id}.jsonl"
        for result in exploration_results:
            patch_candidate = result.get("patch_candidate", {})
            if not patch_candidate.get("diff"):
                continue
            record = self.provenance_recorder.build_record(
                task_id=task_id,
                branch_id=result.get("branch_id", ""),
                localization_result=localization_result,
                patch_candidate=patch_candidate,
                validation_result={
                    "syntax_valid": patch_candidate.get("syntax_valid", False),
                    "constraint_results": patch_candidate.get("constraint_results", []),
                    "validation_errors": patch_candidate.get("validation_errors", []),
                },
                sandbox_apply={
                    "patch_apply_status": result.get("execution", {}).get("metadata", {}).get("patch_apply_status", ""),
                    "rollback_status": result.get("execution", {}).get("metadata", {}).get("rollback_status", ""),
                    "execution_mode": result.get("execution", {}).get("metadata", {}).get("execution_mode", ""),
                    "trust_level": result.get("execution", {}).get("metadata", {}).get("trust_level", ""),
                },
            )
            self.provenance_recorder.append(str(artifact_path), record)
            records.append(record)
        return records
