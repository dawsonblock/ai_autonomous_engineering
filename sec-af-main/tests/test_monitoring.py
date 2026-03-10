"""Tests for continuous monitoring baseline and regression detection."""

import tempfile
from datetime import UTC, datetime
from pathlib import Path

from sec_af.monitoring import compare_with_baseline, load_baseline, save_baseline
from sec_af.schemas.hunt import FindingType, Severity
from sec_af.schemas.output import SecurityAuditResult
from sec_af.schemas.prove import EvidenceLevel, Location, Verdict, VerifiedFinding


def _make_finding(
    fingerprint: str = "fp-1",
    title: str = "Test Finding",
    severity: Severity = Severity.HIGH,
    cwe_id: str = "CWE-89",
) -> VerifiedFinding:
    return VerifiedFinding(
        fingerprint=fingerprint,
        title=title,
        description="Test",
        finding_type=FindingType.SAST,
        cwe_id=cwe_id,
        cwe_name="SQL Injection",
        verdict=Verdict.CONFIRMED,
        evidence_level=EvidenceLevel.REACHABILITY_CONFIRMED,
        rationale="Test",
        severity=severity,
        exploitability_score=7.5,
        location=Location(file_path="app.py", start_line=10, end_line=15),
        sarif_rule_id="sec-af/sast/cwe-89",
        sarif_security_severity=7.5,
    )


def _make_result(findings: list[VerifiedFinding] | None = None, commit: str = "abc123") -> SecurityAuditResult:
    if findings is None:
        findings = [_make_finding()]
    return SecurityAuditResult(
        repository="https://github.com/test/repo",
        commit_sha=commit,
        branch="main",
        timestamp=datetime(2025, 1, 15, tzinfo=UTC),
        depth_profile="standard",
        strategies_used=["injection"],
        provider="harness",
        findings=findings,
        attack_chains=[],
        total_raw_findings=len(findings),
        confirmed=len(findings),
        likely=0,
        inconclusive=0,
        not_exploitable=0,
        noise_reduction_pct=0.0,
        by_severity={},
        duration_seconds=10.0,
        agent_invocations=5,
        cost_usd=0.05,
        cost_breakdown={},
        sarif="",
    )


def test_save_and_load_baseline():
    result = _make_result()
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        path = f.name
    save_baseline(result, path)
    loaded = load_baseline(path)
    assert loaded["commit_sha"] == "abc123"
    assert len(loaded["findings"]) == 1
    Path(path).unlink()


def test_compare_detects_new_finding():
    baseline_result = _make_result(
        findings=[_make_finding(fingerprint="fp-1")],
        commit="old",
    )
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        path = f.name
    save_baseline(baseline_result, path)
    current = _make_result(
        findings=[_make_finding(fingerprint="fp-1"), _make_finding(fingerprint="fp-2", title="New Bug")],
        commit="new",
    )
    result = compare_with_baseline(current, path)
    assert result.regression_detected is True
    assert len(result.new_findings) == 1
    assert result.new_findings[0].finding_title == "New Bug"
    assert result.unchanged_count == 1
    Path(path).unlink()


def test_compare_detects_fixed_finding():
    baseline_result = _make_result(
        findings=[_make_finding(fingerprint="fp-1"), _make_finding(fingerprint="fp-2")],
        commit="old",
    )
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        path = f.name
    save_baseline(baseline_result, path)
    current = _make_result(
        findings=[_make_finding(fingerprint="fp-1")],
        commit="new",
    )
    result = compare_with_baseline(current, path)
    assert result.regression_detected is False
    assert len(result.fixed_findings) == 1
    assert result.fixed_findings[0].status == "fixed"
    Path(path).unlink()


def test_compare_no_regression():
    baseline_result = _make_result(findings=[_make_finding(fingerprint="fp-1")], commit="old")
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        path = f.name
    save_baseline(baseline_result, path)
    current = _make_result(findings=[_make_finding(fingerprint="fp-1")], commit="new")
    result = compare_with_baseline(current, path)
    assert result.regression_detected is False
    assert len(result.new_findings) == 0
    assert result.unchanged_count == 1
    Path(path).unlink()
