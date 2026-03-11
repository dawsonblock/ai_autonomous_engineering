from __future__ import annotations

from fastapi import APIRouter, Depends

from aae.contracts.dashboard import SystemDiagnostic
from aae.dashboard_api.deps import get_runtime_manager
from aae.dashboard_api.runtime_manager import RuntimeManager

router = APIRouter(prefix="/api/health", tags=["health"])


@router.get("", response_model=list[SystemDiagnostic])
async def health(manager: RuntimeManager = Depends(get_runtime_manager)):
    return await manager.diagnostics()
