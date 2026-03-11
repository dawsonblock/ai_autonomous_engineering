from __future__ import annotations

import asyncio
import json

from fastapi import APIRouter, Depends
from fastapi.responses import StreamingResponse

from aae.dashboard_api.deps import get_runtime_manager
from aae.dashboard_api.runtime_manager import RuntimeManager

router = APIRouter(tags=["events"])


def _event_stream(manager: RuntimeManager, queue: asyncio.Queue[dict], workflow_id: str | None = None):
    async def generator():
        try:
            yield ": connected\n\n"
            while True:
                payload = await queue.get()
                if payload.get("__close__"):
                    break
                yield "data: %s\n\n" % json.dumps(payload, sort_keys=True)
        finally:
            manager.unsubscribe(queue, workflow_id=workflow_id)

    return generator()


@router.get("/api/events/stream")
async def stream_all_events(manager: RuntimeManager = Depends(get_runtime_manager)):
    await manager.start()
    queue = manager.subscribe()
    return StreamingResponse(_event_stream(manager, queue), media_type="text/event-stream")


@router.get("/api/workflows/{workflow_id}/events/stream")
async def stream_workflow_events(workflow_id: str, manager: RuntimeManager = Depends(get_runtime_manager)):
    await manager.start()
    queue = manager.subscribe(workflow_id=workflow_id)
    return StreamingResponse(_event_stream(manager, queue, workflow_id=workflow_id), media_type="text/event-stream")
