from __future__ import annotations

import json
from collections import defaultdict
from typing import Awaitable, Callable, DefaultDict, Dict, List, Optional

from aae.contracts.workflow import EventEnvelope
from aae.events.event_logger import EventLogger

EventListener = Callable[[EventEnvelope], Awaitable[None]]


class EventBus:
    def __init__(
        self,
        logger: Optional[EventLogger] = None,
        redis_url: str | None = None,
    ) -> None:
        self.logger = logger
        self.redis_url = redis_url
        self.listeners: DefaultDict[str, List[EventListener]] = defaultdict(list)
        self._redis = None
        self._transport_mode = "memory"

    @property
    def transport_mode(self) -> str:
        return self._transport_mode

    async def start(self) -> None:
        if not self.redis_url:
            return
        try:
            from redis import asyncio as redis_asyncio
        except ImportError:
            self._transport_mode = "memory"
            return
        self._redis = redis_asyncio.from_url(self.redis_url)
        self._transport_mode = "redis"

    async def close(self) -> None:
        if self._redis is not None:
            await self._redis.close()

    def subscribe(self, event_type: str, listener: EventListener) -> None:
        self.listeners[event_type].append(listener)

    async def publish(self, event: EventEnvelope, persist: bool = True) -> None:
        if persist and self.logger is not None:
            await self.logger.append(event)
        if self._redis is not None:
            await self._redis.publish("aae:events", json.dumps(event.model_dump(mode="json")))
        targets = list(self.listeners.get(event.event_type, [])) + list(
            self.listeners.get("*", [])
        )
        if targets:
            await self._dispatch(targets, event)

    async def _dispatch(self, targets: List[EventListener], event: EventEnvelope) -> None:
        for listener in targets:
            await listener(event)
