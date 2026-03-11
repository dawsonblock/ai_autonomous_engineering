from __future__ import annotations

from fastapi import APIRouter, Depends, HTTPException

from aae.contracts.dashboard import (
    DashboardWorkflowDetail,
    DashboardWorkflowLaunchRequest,
    DashboardWorkflowSummary,
)
from aae.dashboard_api.deps import get_runtime_manager
from aae.dashboard_api.runtime_manager import RuntimeManager

router = APIRouter(prefix="/api/workflows", tags=["workflows"])


@router.get("", response_model=list[DashboardWorkflowSummary])
async def list_workflows(manager: RuntimeManager = Depends(get_runtime_manager)):
    await manager.start()
    return [DashboardWorkflowSummary.model_validate(item) for item in manager.list_workflows()]


@router.post("", response_model=DashboardWorkflowDetail)
async def launch_workflow(
    request: DashboardWorkflowLaunchRequest,
    manager: RuntimeManager = Depends(get_runtime_manager),
):
    detail = await manager.launch_workflow(request)
    return DashboardWorkflowDetail.model_validate(
        {
            "summary": detail,
            "launch_request": detail.get("launch_request", {}),
            "events": detail.get("events", []),
            "memory_snapshot": detail.get("memory_snapshot", {}),
            "artifacts": detail.get("artifacts", {}),
            "planner": detail.get("planner", {}),
        }
    )


@router.get("/{workflow_id}", response_model=DashboardWorkflowDetail)
async def get_workflow_detail(
    workflow_id: str,
    manager: RuntimeManager = Depends(get_runtime_manager),
):
    await manager.start()
    detail = manager.get_workflow_detail(workflow_id)
    if detail is None:
        raise HTTPException(status_code=404, detail="workflow not found")
    return DashboardWorkflowDetail.model_validate(
        {
            "summary": detail,
            "launch_request": detail.get("launch_request", {}),
            "events": detail.get("events", []),
            "memory_snapshot": detail.get("memory_snapshot", {}),
            "artifacts": detail.get("artifacts", {}),
            "planner": detail.get("planner", {}),
        }
    )


@router.post("/{workflow_id}/cancel")
async def cancel_workflow(
    workflow_id: str,
    manager: RuntimeManager = Depends(get_runtime_manager),
):
    cancelled = await manager.cancel_workflow(workflow_id)
    if not cancelled:
        raise HTTPException(status_code=404, detail="workflow not found or not active")
    return {"workflow_id": workflow_id, "cancelled": True}


@router.post("/{workflow_id}/rerun", response_model=DashboardWorkflowDetail)
async def rerun_workflow(
    workflow_id: str,
    manager: RuntimeManager = Depends(get_runtime_manager),
):
    try:
        detail = await manager.rerun_workflow(workflow_id)
    except KeyError as exc:
        raise HTTPException(status_code=404, detail=str(exc)) from exc
    return DashboardWorkflowDetail.model_validate(
        {
            "summary": detail,
            "launch_request": detail.get("launch_request", {}),
            "events": detail.get("events", []),
            "memory_snapshot": detail.get("memory_snapshot", {}),
            "artifacts": detail.get("artifacts", {}),
            "planner": detail.get("planner", {}),
        }
    )
