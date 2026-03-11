from __future__ import annotations

from typing import Any


class ExperimentManager:
    def propose(self, strategy_profile: dict[str, Any], tool_profile: dict[str, Any]) -> dict[str, Any]:
        return {
            "planner": {
                "preferred_strategies": strategy_profile.get("preferred_strategies", []),
            },
            "tools": tool_profile.get("tool_ranking", []),
        }
