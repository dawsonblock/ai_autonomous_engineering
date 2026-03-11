from __future__ import annotations

from collections import defaultdict
from typing import Any


class ToolEvaluator:
    def summarize(self, trajectories: list[dict[str, Any]]) -> dict[str, Any]:
        tool_stats = defaultdict(lambda: {"uses": 0, "successes": 0})
        for row in trajectories:
            tool = str(row.get("tool") or row.get("strategy") or "unknown")
            tool_stats[tool]["uses"] += 1
            if row.get("success") or row.get("fixed"):
                tool_stats[tool]["successes"] += 1
        ranking = []
        for tool, stats in tool_stats.items():
            ranking.append(
                {
                    "tool": tool,
                    "uses": stats["uses"],
                    "success_rate": round(stats["successes"] / stats["uses"], 3) if stats["uses"] else 0.0,
                }
            )
        ranking.sort(key=lambda item: (item["success_rate"], item["uses"]), reverse=True)
        return {"tool_ranking": ranking}
