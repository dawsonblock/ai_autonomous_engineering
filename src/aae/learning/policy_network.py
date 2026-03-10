from __future__ import annotations

from collections import defaultdict


class PolicyNetwork:
    def __init__(self) -> None:
        self.weights = defaultdict(lambda: defaultdict(float))

    def fit(self, dataset: list[dict]) -> "PolicyNetwork":
        counts = defaultdict(lambda: defaultdict(float))
        for row in dataset:
            strategy = row.get("strategy") or row.get("tool") or "graph_query"
            task_type = row.get("task_type", "unknown")
            reward = float(row.get("reward", row.get("success", 0)))
            counts[task_type][strategy] += max(0.1, 1.0 + reward)
        self.weights = counts
        return self

    def predict_ranked(self, task_type: str) -> dict[str, float]:
        strategy_weights = dict(self.weights.get(task_type, {}))
        total = sum(strategy_weights.values())
        if total <= 0:
            return {}
        return dict(sorted(((name, value / total) for name, value in strategy_weights.items()), key=lambda item: item[1], reverse=True))
