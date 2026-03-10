from __future__ import annotations

import pytest

from sec_af.agents import hunt as hunt_module
from sec_af.schemas.hunt import Confidence, FindingType, HuntResult, HuntStrategy, RawFinding, Severity
from sec_af.schemas.recon import (
    ArchitectureMap,
    ConfigReport,
    DataFlowMap,
    DependencyReport,
    ReconResult,
    SecurityContext,
)


class _App:
    async def harness(self, prompt: str, *, schema: object = None, cwd: str | None = None, **kwargs: object) -> object:
        _ = (prompt, schema, cwd, kwargs)
        return {}


def _recon_result() -> ReconResult:
    return ReconResult(
        architecture=ArchitectureMap(),
        data_flows=DataFlowMap(),
        dependencies=DependencyReport(),
        config=ConfigReport(),
        security_context=SecurityContext(auth_model="session", auth_details="cookie"),
    )


def _finding(file_path: str, suffix: str) -> RawFinding:
    return RawFinding(
        id=f"id-{suffix}",
        hunter_strategy="injection",
        title=f"title-{suffix}",
        description="desc",
        finding_type=FindingType.SAST,
        cwe_id="CWE-89",
        cwe_name="SQL Injection",
        file_path=file_path,
        start_line=1,
        end_line=1,
        code_snippet="query",
        estimated_severity=Severity.HIGH,
        confidence=Confidence.MEDIUM,
        fingerprint=f"fp-{suffix}",
    )


@pytest.mark.asyncio
async def test_run_hunt_filters_findings_by_include_paths(monkeypatch) -> None:
    include_capture: dict[str, list[str] | None] = {}

    async def fake_runner(**kwargs):
        include_capture["value"] = kwargs.get("include_paths")
        return [_finding("src/keep.py", "keep"), _finding("src/drop.py", "drop")]

    async def fake_deduplicate(all_findings, *_args, **_kwargs):
        return HuntResult(findings=list(all_findings))

    monkeypatch.setattr(hunt_module, "_select_strategies", lambda _depth: [HuntStrategy.INJECTION])
    monkeypatch.setitem(hunt_module._STRATEGY_RUNNERS, HuntStrategy.INJECTION, fake_runner)
    monkeypatch.setattr(hunt_module, "deduplicate_and_correlate", fake_deduplicate)

    result = await hunt_module.run_hunt(
        app=_App(),
        repo_path=".",
        recon_result=_recon_result(),
        depth="standard",
        include_paths=["src/keep.py"],
    )

    assert include_capture["value"] == ["src/keep.py"]
    assert [finding.file_path for finding in result.findings] == ["src/keep.py"]
    assert result.total_raw == 2
