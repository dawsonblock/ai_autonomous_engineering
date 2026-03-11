from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict


class PatchProvenanceRecorder:
    def build_record(
        self,
        task_id: str,
        branch_id: str,
        localization_result: dict[str, Any],
        patch_candidate: dict[str, Any],
        validation_result: dict[str, Any],
        sandbox_apply: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        return {
            "task_id": task_id,
            "branch_id": branch_id,
            "localization_result": localization_result,
            "selected_spans": list(patch_candidate.get("target_spans", [])),
            "patch_diff": patch_candidate.get("diff", ""),
            "validation_result": validation_result,
            "sandbox_apply": sandbox_apply or {},
        }

    def append(self, artifact_path: str, record: Dict[str, Any]) -> str:
        path = Path(artifact_path)
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(record, sort_keys=True) + "\n")
        return str(path)
