from __future__ import annotations

from collections import defaultdict
from typing import Any


class StrategyOptimizer:
    def rank(self, evaluation_records: list[dict[str, Any]], patch_provenance: list[dict[str, Any]]) -> dict[str, Any]:
        outcomes = defaultdict(lambda: {"success": 0, "total": 0})
        provenance_by_branch = {record.get("branch_id", ""): record for record in patch_provenance}
        for record in evaluation_records:
            branch_id = record.get("selected_branch_id", "")
            provenance = provenance_by_branch.get(branch_id, {})
            validation = provenance.get("validation_result", {})
            strategy = provenance.get("localization_result", {}).get("root_cause_symbol", "") or branch_id or "default"
            outcomes[strategy]["total"] += 1
            if record.get("fixed"):
                outcomes[strategy]["success"] += 1
            if validation.get("syntax_valid"):
                outcomes[strategy]["success"] += 0
        ranked = []
        for strategy, counts in outcomes.items():
            success_rate = counts["success"] / counts["total"] if counts["total"] else 0.0
            ranked.append({"strategy": strategy, "success_rate": round(success_rate, 3), "total_runs": counts["total"]})
        ranked.sort(key=lambda item: (item["success_rate"], item["total_runs"]), reverse=True)
        return {"preferred_strategies": ranked[:5]}
