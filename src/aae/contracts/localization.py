from __future__ import annotations

from typing import Any, Dict, List

from pydantic import BaseModel, Field


class FailureEvidence(BaseModel):
    source: str
    file_path: str = ""
    symbol: str = ""
    line: int = 0
    weight: float = 0.0
    metadata: Dict[str, Any] = Field(default_factory=dict)


class SuspiciousnessScore(BaseModel):
    location_id: str
    file_path: str
    symbol: str = ""
    line: int = 0
    score: float = 0.0
    components: Dict[str, float] = Field(default_factory=dict)


class SuspiciousLocation(BaseModel):
    file_path: str
    symbol: str = ""
    start_line: int = 0
    end_line: int = 0
    confidence: float = 0.0
    evidence_sources: List[str] = Field(default_factory=list)
    score_components: Dict[str, float] = Field(default_factory=dict)


class LocalizationResult(BaseModel):
    suspicious_locations: List[SuspiciousLocation] = Field(default_factory=list)
    evidence: List[FailureEvidence] = Field(default_factory=list)
    summary: Dict[str, Any] = Field(default_factory=dict)
