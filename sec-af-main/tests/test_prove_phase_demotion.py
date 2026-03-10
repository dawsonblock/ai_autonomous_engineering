from __future__ import annotations

from typing import Any

import pytest

from sec_af.reasoners import phases
from sec_af.schemas.hunt import Confidence, FindingType, RawFinding, Severity


def _raw_finding() -> RawFinding:
    return RawFinding(
        id="raw-1",
        hunter_strategy="injection",
        title="Potential SQL injection",
        description="Potential injection from request parameter",
        finding_type=FindingType.SAST,
        cwe_id="CWE-89",
        cwe_name="SQL Injection",
        file_path="src/users.py",
        start_line=10,
        end_line=10,
        code_snippet='cursor.execute(f"SELECT * FROM users WHERE id = {user_id}")',
        estimated_severity=Severity.HIGH,
        confidence=Confidence.HIGH,
        related_files=[],
        fingerprint="fp-1",
    )


class _RouterStub:
    def __init__(self, verifier_payload: dict[str, Any]):
        self._verifier_payload = verifier_payload
        self.notes: list[str] = []

    async def call(self, name: str, **kwargs: object) -> dict[str, Any]:
        if name.endswith("run_verifier"):
            return {"output": self._verifier_payload}
        if name.endswith("run_deduplicator"):
            return {"output": {"findings": [], "chains": []}}
        raise AssertionError(f"unexpected call: {name}, kwargs={kwargs}")

    def note(self, message: str, tags: list[str] | None = None) -> None:
        _ = tags
        self.notes.append(message)


@pytest.mark.asyncio
async def test_prove_phase_demotes_unverified_verdict(monkeypatch: pytest.MonkeyPatch) -> None:
    finding = _raw_finding()
    router = _RouterStub(
        {
            "id": finding.id,
            "fingerprint": finding.fingerprint,
            "title": finding.title,
            "description": finding.description,
            "finding_type": finding.finding_type.value,
            "cwe_id": finding.cwe_id,
            "cwe_name": finding.cwe_name,
            "verdict": "unverified",
            "evidence_level": 1,
            "rationale": "Could not fully verify",
            "severity": finding.estimated_severity.value,
            "exploitability_score": 0.0,
            "location": {
                "file_path": finding.file_path,
                "start_line": finding.start_line,
                "end_line": finding.end_line,
                "code_snippet": finding.code_snippet,
            },
            "sarif_rule_id": "sec-af/sast/sql-injection",
            "sarif_security_severity": 0.0,
        }
    )
    monkeypatch.setattr(phases, "_runtime_router", router)

    result = await phases.prove_phase(
        repo_path="/tmp/repo",
        hunt_result={
            "findings": [finding.model_dump()],
            "chains": [],
            "total_raw": 1,
            "deduplicated_count": 1,
            "chain_count": 0,
            "strategies_run": ["injection"],
            "hunt_duration_seconds": 0.0,
        },
    )

    assert result["drop_summary"]["demoted_total"] == 1
    assert result["verified"][0]["verdict"] == "inconclusive"
    assert result["verified"][0]["drop_reason"] == "verdict_unverified"
    assert "low_confidence" in result["verified"][0]["tags"]


@pytest.mark.asyncio
async def test_prove_phase_demotes_parse_failure(monkeypatch: pytest.MonkeyPatch) -> None:
    finding = _raw_finding()
    router = _RouterStub({"title": "malformed"})
    monkeypatch.setattr(phases, "_runtime_router", router)

    result = await phases.prove_phase(
        repo_path="/tmp/repo",
        hunt_result={
            "findings": [finding.model_dump()],
            "chains": [],
            "total_raw": 1,
            "deduplicated_count": 1,
            "chain_count": 0,
            "strategies_run": ["injection"],
            "hunt_duration_seconds": 0.0,
        },
    )

    assert result["drop_summary"]["demoted_total"] == 1
    assert result["verified"][0]["verdict"] == "inconclusive"
    assert result["verified"][0]["drop_reason"] == "schema_parse_failure"
