from __future__ import annotations

import json
import statistics
from pathlib import Path
from typing import Any


class MetricsReporter:
    def summarize(self, records: list[dict[str, Any]]) -> dict[str, Any]:
        total = len(records)
        fixed = [record for record in records if record.get("fixed")]
        patch_sizes = [int(record.get("patch_size", 0) or 0) for record in records if record.get("patch_size") is not None]
        success_runtimes = [float(record.get("runtime_cost_s", 0.0) or 0.0) for record in fixed]
        total_selected_tests = sum(max(1, int(record.get("selected_test_count", 0) or 0)) for record in records)
        total_regressions = sum(int(record.get("regression_count", 0) or 0) for record in records)
        return {
            "case_count": total,
            "success_count": len(fixed),
            "fix_rate": round(len(fixed) / total, 3) if total else 0.0,
            "median_patch_size": int(statistics.median(patch_sizes)) if patch_sizes else 0,
            "runtime_per_success_s": round(sum(success_runtimes) / len(success_runtimes), 3) if success_runtimes else 0.0,
            "runtime_per_success_min": round((sum(success_runtimes) / len(success_runtimes)) / 60.0, 3) if success_runtimes else 0.0,
            "regression_rate": round(total_regressions / total_selected_tests, 3) if total_selected_tests else 0.0,
        }

    def write_report(self, path: str | Path, payload: dict[str, Any]) -> str:
        report_path = Path(path)
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
        return str(report_path)
