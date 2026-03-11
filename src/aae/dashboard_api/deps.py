from __future__ import annotations

from aae.dashboard_api.runtime_manager import RuntimeManager

_manager: RuntimeManager | None = None


def set_runtime_manager(manager: RuntimeManager | None) -> None:
    global _manager
    _manager = manager


def get_runtime_manager() -> RuntimeManager:
    global _manager
    if _manager is None:
        _manager = RuntimeManager()
    return _manager
