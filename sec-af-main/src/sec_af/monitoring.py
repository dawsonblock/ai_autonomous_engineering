"""Continuous monitoring: baseline storage and regression detection.

Compares current scan results against a stored baseline to identify
new vulnerabilities (regressions) and fixed issues.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import TYPE_CHECKING, TypedDict, cast

if TYPE_CHECKING:
    from .schemas.output import MonitoringResult, RegressionFinding, SecurityAuditResult


class BaselineFinding(TypedDict):
    id: str
    fingerprint: str
    title: str
    severity: str
    cwe_id: str
    verdict: str
    file_path: str
    start_line: int


class BaselineData(TypedDict):
    commit_sha: str
    timestamp: str
    findings: list[BaselineFinding]


def save_baseline(result: SecurityAuditResult, path: str) -> None:
    """Save scan results as baseline for future comparison."""
    baseline_data = {
        "commit_sha": result.commit_sha,
        "timestamp": result.timestamp.isoformat(),
        "findings": [
            {
                "id": f.id,
                "fingerprint": f.fingerprint,
                "title": f.title,
                "severity": f.severity.value,
                "cwe_id": f.cwe_id,
                "verdict": f.verdict.value,
                "file_path": f.location.file_path,
                "start_line": f.location.start_line,
            }
            for f in result.findings
        ],
    }
    _ = Path(path).write_text(json.dumps(baseline_data, indent=2), encoding="utf-8")


def load_baseline(path: str) -> BaselineData:
    """Load baseline scan data from file."""
    return cast("BaselineData", json.loads(Path(path).read_text(encoding="utf-8")))


def compare_with_baseline(
    current: SecurityAuditResult,
    baseline_path: str,
) -> MonitoringResult:
    """Compare current scan results against stored baseline."""
    from .schemas.output import MonitoringResult, RegressionFinding

    baseline = load_baseline(baseline_path)
    baseline_fingerprints = {f["fingerprint"] for f in baseline["findings"]}
    baseline_by_fp = {f["fingerprint"]: f for f in baseline["findings"]}

    current_fingerprints = {f.fingerprint for f in current.findings}

    new_findings: list[RegressionFinding] = []
    for finding in current.findings:
        if finding.fingerprint not in baseline_fingerprints:
            new_findings.append(
                RegressionFinding(
                    finding_title=finding.title,
                    finding_id=finding.id,
                    severity=finding.severity.value,
                    cwe_id=finding.cwe_id,
                    status="new",
                )
            )

    fixed_findings: list[RegressionFinding] = []
    for fp, bf in baseline_by_fp.items():
        if fp not in current_fingerprints:
            fixed_findings.append(
                RegressionFinding(
                    finding_title=bf["title"],
                    finding_id=bf["id"],
                    severity=bf["severity"],
                    cwe_id=bf["cwe_id"],
                    status="fixed",
                )
            )

    unchanged_count = len(baseline_fingerprints & current_fingerprints)

    return MonitoringResult(
        baseline_commit=baseline.get("commit_sha", "unknown"),
        current_commit=current.commit_sha,
        new_findings=new_findings,
        fixed_findings=fixed_findings,
        unchanged_count=unchanged_count,
        regression_detected=len(new_findings) > 0,
    )
