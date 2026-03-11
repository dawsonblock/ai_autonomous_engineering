from __future__ import annotations

from fastapi import APIRouter, Depends

from aae.contracts.dashboard import DashboardBenchmarkRunRequest, DashboardBenchmarkSummary
from aae.dashboard_api.deps import get_runtime_manager
from aae.dashboard_api.runtime_manager import RuntimeManager

router = APIRouter(prefix="/api/benchmarks", tags=["benchmarks"])


@router.get("")
async def list_benchmarks(manager: RuntimeManager = Depends(get_runtime_manager)):
    await manager.start()
    latest = manager.latest_benchmark_report()
    return {
        "latest": DashboardBenchmarkSummary.model_validate(latest) if latest else None,
        "reports": [DashboardBenchmarkSummary.model_validate(item) for item in manager.list_benchmark_reports()],
    }


@router.post("/run", response_model=DashboardBenchmarkSummary)
async def run_benchmarks(
    request: DashboardBenchmarkRunRequest,
    manager: RuntimeManager = Depends(get_runtime_manager),
):
    report = await manager.run_benchmarks(request)
    return DashboardBenchmarkSummary.model_validate(report)
