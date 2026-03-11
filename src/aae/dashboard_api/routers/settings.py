from __future__ import annotations

from fastapi import APIRouter, Depends

from aae.contracts.dashboard import RuntimeOverrideProfile
from aae.dashboard_api.deps import get_runtime_manager
from aae.dashboard_api.runtime_manager import RuntimeManager

router = APIRouter(prefix="/api/settings", tags=["settings"])


@router.get("/runtime-overrides", response_model=RuntimeOverrideProfile)
async def get_runtime_overrides(manager: RuntimeManager = Depends(get_runtime_manager)):
    await manager.start()
    return manager.get_runtime_overrides()


@router.put("/runtime-overrides", response_model=RuntimeOverrideProfile)
async def update_runtime_overrides(
    profile: RuntimeOverrideProfile,
    manager: RuntimeManager = Depends(get_runtime_manager),
):
    await manager.start()
    return manager.update_runtime_overrides(profile)
