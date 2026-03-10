from __future__ import annotations

from aae.patching.git_ops.git_patch_applier import GitPatchApplier
from aae.patching.git_ops.git_safety import GitSafety
from aae.patching.git_ops.rollback_manager import RollbackManager


class MultiFileEditor:
    def __init__(
        self,
        git_safety: GitSafety | None = None,
        patch_applier: GitPatchApplier | None = None,
        rollback_manager: RollbackManager | None = None,
    ) -> None:
        self.git_safety = git_safety or GitSafety()
        self.patch_applier = patch_applier or GitPatchApplier(self.git_safety)
        self.rollback_manager = rollback_manager or RollbackManager()

    def begin_edit(self, repo_path: str) -> dict:
        return self.git_safety.ensure_git_snapshot(repo_path)

    def apply_multi_file_patch(self, repo_path: str, diffs: list[str]) -> dict:
        session = self.begin_edit(repo_path)
        applied = []
        for diff in diffs:
            result = self.patch_applier.apply_patch(repo_path, diff)
            if not result["applied"]:
                rollback = self.rollback(repo_path)
                return {
                    "applied": False,
                    "results": applied + [result],
                    "rollback": rollback,
                    "session": session,
                }
            applied.append(result)
        return {
            "applied": True,
            "results": applied,
            "rollback": {"rolled_back": False},
            "session": session,
        }

    def rollback(self, repo_path: str) -> dict:
        return self.rollback_manager.rollback(repo_path)
