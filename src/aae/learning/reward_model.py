from __future__ import annotations


class RewardModel:
    def score(self, row: dict) -> float:
        success = float(row.get("success", 0))
        runtime_cost = float(row.get("runtime_cost_s", 0.0))
        regression_count = float(row.get("regression_count", 0))
        risk_score = float(row.get("risk_score", 0.0))
        return round((success * 2.0) - (runtime_cost * 0.01) - (regression_count * 0.5) - (risk_score * 0.3), 3)
