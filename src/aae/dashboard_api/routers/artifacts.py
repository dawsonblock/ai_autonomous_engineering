from __future__ import annotations

import json
from pathlib import Path

from fastapi import APIRouter, Depends, HTTPException, Query

from aae.dashboard_api.deps import get_runtime_manager
from aae.dashboard_api.runtime_manager import RuntimeManager

router = APIRouter(prefix="/api/artifacts", tags=["artifacts"])


@router.get("/workflows/{workflow_id}")
async def get_workflow_artifacts(
    workflow_id: str,
    manager: RuntimeManager = Depends(get_runtime_manager),
):
    await manager.start()
    detail = manager.get_workflow_detail(workflow_id)
    if detail is None:
        raise HTTPException(status_code=404, detail="workflow not found")
    return {
        "workflow_id": workflow_id,
        "events": detail.get("events", []),
        "patch_provenance": detail.get("patch_provenance", []),
        "memory_snapshot": detail.get("memory_snapshot", {}),
        "artifacts": detail.get("artifacts", {}),
    }


@router.get("/file")
async def read_artifact_file(
    path: str = Query(..., description="Absolute path under the artifacts directory"),
    manager: RuntimeManager = Depends(get_runtime_manager),
):
    await manager.start()
    target = Path(path).resolve()
    artifacts_root = manager.artifacts_dir.resolve()
    try:
        target.relative_to(artifacts_root)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail="path must be inside artifacts directory") from exc
    if not target.exists():
        raise HTTPException(status_code=404, detail="artifact not found")
    if target.suffix in {".json", ".jsonl"}:
        text = target.read_text(encoding="utf-8")
        if target.suffix == ".json":
            return {"path": str(target), "content": json.loads(text)}
        return {"path": str(target), "content": [json.loads(line) for line in text.splitlines() if line.strip()]}
    return {"path": str(target), "content": target.read_text(encoding="utf-8", errors="ignore")}
