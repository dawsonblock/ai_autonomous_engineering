from __future__ import annotations

from typing import Any


DEFAULT_THRESHOLDS = {
    "fix_rate": 0.35,
    "median_patch_size": 20,
    "runtime_per_success_min": 20.0,
    "regression_rate": 0.05,
}


class RegressionTests:
    def evaluate(self, metrics: dict[str, Any], thresholds: dict[str, float] | None = None) -> dict[str, Any]:
        thresholds = thresholds or DEFAULT_THRESHOLDS
        breaches = []
        if float(metrics.get("fix_rate", 0.0)) < thresholds["fix_rate"]:
            breaches.append("fix_rate")
        if float(metrics.get("median_patch_size", 0.0)) > thresholds["median_patch_size"]:
            breaches.append("median_patch_size")
        if float(metrics.get("runtime_per_success_min", 0.0)) > thresholds["runtime_per_success_min"]:
            breaches.append("runtime_per_success_min")
        if float(metrics.get("regression_rate", 0.0)) > thresholds["regression_rate"]:
            breaches.append("regression_rate")
        return {
            "passed": not breaches,
            "breaches": breaches,
            "thresholds": thresholds,
        }
