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


class PatchPlan(BaseModel):
    id: str
    summary: str
    confidence: float = 0.0
    target_files: List[str] = Field(default_factory=list)
    strategy: str = ""


class PatchCandidate(BaseModel):
    plan_id: str
    diff: str = ""
    changed_files: List[str] = Field(default_factory=list)
    confidence: float = 0.0
    risk_score: float = 0.0
    predicted_test_count: int = 0
    impact_size: int = 0


class PatchReviewResult(BaseModel):
    accept: bool = False
    risks: List[str] = Field(default_factory=list)
    followups: List[str] = Field(default_factory=list)


class FailureAnalysisResult(BaseModel):
    failure_type: str = ""
    suspected_file: str = ""
    reason: str = ""
