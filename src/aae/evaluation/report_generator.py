from __future__ import annotations

from pathlib import Path
from typing import Any


class ReportGenerator:
    def write_markdown(self, path: str | Path, report: dict[str, Any]) -> str:
        output = Path(path)
        output.parent.mkdir(parents=True, exist_ok=True)
        metrics = report.get("metrics", {})
        regression = report.get("regression_summary", {})
        lines = [
            "# Benchmark Report",
            "",
            "## Metrics",
            "",
            "- Strict fix rate: %s" % metrics.get("strict_fix_rate", 0.0),
            "- Raw fix rate: %s" % metrics.get("raw_fix_rate", 0.0),
            "- Degraded runs: %s" % metrics.get("degraded_run_count", 0),
            "- Median patch size: %s" % metrics.get("median_patch_size", 0),
            "- Runtime per strict success (min): %s" % metrics.get("runtime_per_success_min", 0.0),
            "- Regression rate: %s" % metrics.get("regression_rate", 0.0),
            "",
            "## Regression Gates",
            "",
            "- Passed: %s" % regression.get("passed", False),
            "- Breaches: %s" % ", ".join(regression.get("breaches", [])) if regression.get("breaches") else "- Breaches: none",
            "",
            "## Cases",
            "",
        ]
        for record in report.get("records", []):
            localization = record.get("localization_metrics", {})
            lines.extend(
                [
                    "### %s" % record.get("case_id", "unknown"),
                    "",
                    "- Fixed (strict): %s" % record.get("fixed", False),
                    "- Branch succeeded: %s" % record.get("branch_succeeded", False),
                    "- Execution mode: %s" % record.get("execution_mode", ""),
                    "- Trust level: %s" % record.get("trust_level", ""),
                    "- Patch size: %s" % record.get("patch_size", 0),
                    "- Selected branch: %s" % record.get("selected_branch_id", ""),
                    "- Localization file_top1: %s" % localization.get("file_top1", False),
                    "- Localization function_top3: %s" % localization.get("function_top3", False),
                    "",
                ]
            )
        output.write_text("\n".join(lines) + "\n", encoding="utf-8")
        return str(output)
