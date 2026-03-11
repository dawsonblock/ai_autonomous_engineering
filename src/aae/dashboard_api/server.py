from __future__ import annotations

import argparse
from pathlib import Path

from contextlib import asynccontextmanager

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse, HTMLResponse, Response

from aae.dashboard_api.deps import get_runtime_manager, set_runtime_manager
from aae.dashboard_api.runtime_manager import RuntimeManager
from aae.dashboard_api.routers import artifacts, benchmarks, events, health, settings, workflows


def create_app(runtime_manager: RuntimeManager | None = None) -> FastAPI:
    if runtime_manager is not None:
        set_runtime_manager(runtime_manager)

    @asynccontextmanager
    async def lifespan(app: FastAPI):
        await get_runtime_manager().start()
        try:
            yield
        finally:
            await get_runtime_manager().close()

    app = FastAPI(title="AAE Local Dashboard", docs_url="/api/docs", redoc_url="/api/redoc", lifespan=lifespan)
    app.add_middleware(
        CORSMiddleware,
        allow_origins=[
            "http://127.0.0.1:4173",
            "http://127.0.0.1:5173",
            "http://localhost:4173",
            "http://localhost:5173",
        ],
        allow_credentials=False,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    app.include_router(workflows.router)
    app.include_router(events.router)
    app.include_router(benchmarks.router)
    app.include_router(artifacts.router)
    app.include_router(settings.router)
    app.include_router(health.router)

    dist_dir = Path(__file__).resolve().parents[3] / "dashboard" / "dist"
    assets_dir = dist_dir / "assets"
    if assets_dir.exists():
        from fastapi.staticfiles import StaticFiles

        app.mount("/assets", StaticFiles(directory=assets_dir), name="dashboard-assets")

    @app.get("/", include_in_schema=False)
    async def serve_root() -> Response:
        return _serve_frontend(dist_dir)

    @app.get("/{full_path:path}", include_in_schema=False)
    async def serve_spa(full_path: str, request: Request) -> Response:
        if full_path.startswith("api/"):
            return HTMLResponse("Not Found", status_code=404)
        candidate = dist_dir / full_path
        if candidate.exists() and candidate.is_file():
            return FileResponse(candidate)
        return _serve_frontend(dist_dir)

    return app


def _serve_frontend(dist_dir: Path) -> Response:
    index_path = dist_dir / "index.html"
    if index_path.exists():
        return FileResponse(index_path)
    return HTMLResponse(
        """
        <html>
          <head><title>AAE Dashboard</title></head>
          <body style="font-family: sans-serif; padding: 2rem;">
            <h1>AAE Dashboard</h1>
            <p>The frontend bundle is missing. Build <code>dashboard/</code> to serve the control center.</p>
          </body>
        </html>
        """,
        status_code=503,
    )


def main() -> None:
    import uvicorn

    parser = argparse.ArgumentParser(description="Launch the local AAE dashboard")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", default=8787, type=int)
    args = parser.parse_args()
    uvicorn.run(create_app(), host=args.host, port=args.port)
