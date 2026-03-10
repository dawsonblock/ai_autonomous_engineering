from __future__ import annotations

from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field


class FailureSignal(BaseModel):
    test_name: str
    exception_type: Optional[str] = None
    exception_message: Optional[str] = None
    file_path: Optional[str] = None
    line_number: Optional[int] = None
    stacktrace: Optional[str] = None
    expected: Optional[str] = None
    actual: Optional[str] = None
    raw_output: Optional[str] = None


class StackFrameRef(BaseModel):
    file_path: str
    function_name: Optional[str] = None
    line_number: Optional[int] = None
    module_name: Optional[str] = None
    in_project: bool = True
    weight: float = 0.0
    resolved_symbol_id: Optional[str] = None


class CoverageRecord(BaseModel):
    test_name: str
    file_path: str
    function_name: Optional[str] = None
    line_hits: List[int] = Field(default_factory=list)


class RankedFile(BaseModel):
    file_path: str
    score: float
    reasons: List[str] = Field(default_factory=list)
    evidence: Dict[str, Any] = Field(default_factory=dict)


class RankedFunction(BaseModel):
    file_path: str
    function_name: str
    score: float
    reasons: List[str] = Field(default_factory=list)
    evidence: Dict[str, Any] = Field(default_factory=dict)


class RankedSpan(BaseModel):
    file_path: str
    start_line: int
    end_line: int
    score: float
    span_type: str
    reasons: List[str] = Field(default_factory=list)
    evidence: Dict[str, Any] = Field(default_factory=dict)


class LocalizationResult(BaseModel):
    files: List[RankedFile]
    functions: List[RankedFunction]
    spans: List[RankedSpan]
    summary: Dict[str, Any] = Field(default_factory=dict)
