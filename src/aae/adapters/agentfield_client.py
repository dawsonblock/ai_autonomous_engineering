from __future__ import annotations

from typing import Any, Dict, Optional

import httpx

from aae.adapters.base import TransientAdapterError


class AgentFieldExecutionError(RuntimeError):
    pass


class AgentFieldClient:
    def __init__(
        self,
        base_url: str,
        api_key: str | None = None,
        poll_interval_s: float = 1.0,
        request_timeout_s: float = 30.0,
        http_client: Optional[httpx.AsyncClient] = None,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.poll_interval_s = poll_interval_s
        self.request_timeout_s = request_timeout_s
        self._client = http_client or httpx.AsyncClient(timeout=request_timeout_s)
        self._owns_client = http_client is None

    async def aclose(self) -> None:
        if self._owns_client:
            await self._client.aclose()

    async def execute(
        self,
        target: str,
        payload: Dict[str, Any],
        timeout_s: float,
    ) -> Dict[str, Any]:
        headers = {}
        if self.api_key:
            headers["X-API-Key"] = self.api_key
        submit = await self._client.post(
            "%s/api/v1/execute/async/%s" % (self.base_url, target),
            headers=headers,
            json={"input": payload},
        )
        submit.raise_for_status()
        body = submit.json()
        execution_id = body.get("execution_id") or body.get("id")
        if not execution_id:
            raise AgentFieldExecutionError("AgentField submit response missing execution_id")

        elapsed = 0.0
        while elapsed <= timeout_s:
            response = await self._client.get(
                "%s/api/v1/executions/%s" % (self.base_url, execution_id),
                headers=headers,
            )
            response.raise_for_status()
            result = response.json()
            status = str(result.get("status", "")).lower()
            if status in {"completed", "succeeded", "success"}:
                return self._unwrap_result(result)
            if status in {"failed", "error", "cancelled"}:
                message = result.get("error") or result.get("detail") or result.get("message") or "execution failed"
                raise AgentFieldExecutionError(str(message))
            await self._sleep()
            elapsed += self.poll_interval_s

        raise TransientAdapterError(
            "timed out waiting for AgentField execution '%s'" % execution_id
        )

    async def _sleep(self) -> None:
        import asyncio

        await asyncio.sleep(self.poll_interval_s)

    def _unwrap_result(self, result: Dict[str, Any]) -> Dict[str, Any]:
        if isinstance(result.get("output"), dict):
            return result["output"]
        if isinstance(result.get("result"), dict):
            return result["result"]
        return result
