from __future__ import annotations

from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field


class MicroAgentInput(BaseModel):
    task: Dict[str, Any] = Field(default_factory=dict)
    context: Dict[str, Any] = Field(default_factory=dict)


class MicroAgentOutput(BaseModel):
    agent_name: str
    payload: Dict[str, Any] = Field(default_factory=dict)


class CandidateFile(BaseModel):
    path: str
    reason: str
    score: float = 0.0


class RepoMapResult(BaseModel):
    candidate_files: List[CandidateFile] = Field(default_factory=list)
    entrypoints: List[str] = Field(default_factory=list)
    tests: List[str] = Field(default_factory=list)
    confidence: float = 0.0


class SymbolLocation(BaseModel):
    name: str
    file: str = ""
    line: int = 0


class SymbolLocatorResult(BaseModel):
    symbols: List[SymbolLocation] = Field(default_factory=list)


class DependencyTraceResult(BaseModel):
    call_chain: List[str] = Field(default_factory=list)
    impacted_tests: List[str] = Field(default_factory=list)
    imported_modules: List[str] = Field(default_factory=list)


class BugLocalizationResult(BaseModel):
    candidate_files: List[CandidateFile] = Field(default_factory=list)
    root_cause_symbol: str = ""
    confidence: float = 0.0


class TestImpactResult(BaseModel):
    tests: List[str] = Field(default_factory=list)
    confidence: float = 0.0


class DependencyRiskResult(BaseModel):
    risk_score: float = 0.0
    risk_reasons: List[str] = Field(default_factory=list)


class RegressionGuardResult(BaseModel):
    accept: bool = False
    boundary_ok: bool = False
    reasons: List[str] = Field(default_factory=list)


class PatchPlan(BaseModel):
    id: str
    summary: str
    confidence: float = 0.0
    target_files: List[str] = Field(default_factory=list)
    strategy: str = ""


class PatchTargetSpan(BaseModel):
    file_path: str
    symbol: str = ""
    start_line: int
    end_line: int


class PatchConstraintResult(BaseModel):
    name: str
    passed: bool
    details: str = ""


class PatchValidationResult(BaseModel):
    syntax_valid: bool = False
    passed: bool = False
    errors: List[str] = Field(default_factory=list)
    constraint_results: List[PatchConstraintResult] = Field(default_factory=list)


class PatchGenerationRequest(BaseModel):
    file_path: str
    symbol: str
    strategy: str = ""
    expected_behavior: str = ""
    target_span: PatchTargetSpan
    semantic_context: Dict[str, Any] = Field(default_factory=dict)
    constraints: Dict[str, Any] = Field(default_factory=dict)


class PatchCandidate(BaseModel):
    plan_id: str
    diff: str = ""
    changed_files: List[str] = Field(default_factory=list)
    confidence: float = 0.0
    risk_score: float = 0.0
    predicted_test_count: int = 0
    impact_size: int = 0
    target_spans: List[PatchTargetSpan] = Field(default_factory=list)
    syntax_valid: bool = False
    constraint_results: List[PatchConstraintResult] = Field(default_factory=list)
    validation_errors: List[str] = Field(default_factory=list)
    changed_symbols: List[str] = Field(default_factory=list)


class PatchReviewResult(BaseModel):
    accept: bool = False
    risks: List[str] = Field(default_factory=list)
    followups: List[str] = Field(default_factory=list)


class FailureAnalysisResult(BaseModel):
    failure_type: str = ""
    suspected_file: str = ""
    reason: str = ""
