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
            "- Fix rate: %s" % metrics.get("fix_rate", 0.0),
            "- Median patch size: %s" % metrics.get("median_patch_size", 0),
            "- Runtime per success (min): %s" % metrics.get("runtime_per_success_min", 0.0),
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
            lines.extend(
                [
                    "### %s" % record.get("case_id", "unknown"),
                    "",
                    "- Fixed: %s" % record.get("fixed", False),
                    "- Patch size: %s" % record.get("patch_size", 0),
                    "- Selected branch: %s" % record.get("selected_branch_id", ""),
                    "",
                ]
            )
        output.write_text("\n".join(lines) + "\n", encoding="utf-8")
        return str(output)
