from __future__ import annotations

from collections import defaultdict


class ToolPolicyModel:
    def __init__(self) -> None:
        self.weights = defaultdict(lambda: defaultdict(float))

    def fit(self, dataset: list[dict]) -> "ToolPolicyModel":
        counts = defaultdict(lambda: defaultdict(float))
        for row in dataset:
            counts[row["task_type"]][row["tool"]] += 1.0 + float(row.get("success", 0))
        self.weights = counts
        return self

    def predict_proba(self, task_type: str) -> dict[str, float]:
        tool_weights = dict(self.weights.get(task_type, {}))
        total = sum(tool_weights.values())
        if total <= 0:
            return {}
        return {tool: value / total for tool, value in tool_weights.items()}
