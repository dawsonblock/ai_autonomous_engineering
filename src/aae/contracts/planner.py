from __future__ import annotations

from typing import Any, Dict, List

from pydantic import BaseModel, Field


class CandidatePlan(BaseModel):
    plan_id: str
    summary: str
    confidence: float = 0.0
    agent_name: str = ""
    changed_files: List[str] = Field(default_factory=list)
    impact_size: int = 0
    predicted_test_count: int = 0
    risk_score: float = 0.0
    diff: str = ""
    risk_reasons: List[str] = Field(default_factory=list)


class JudgeScoreBreakdown(BaseModel):
    plan_id: str
    total_score: float
    components: Dict[str, float] = Field(default_factory=dict)


class ConsensusDecision(BaseModel):
    selected_plan_id: str = ""
    shortlisted_plan_ids: List[str] = Field(default_factory=list)
    score_breakdowns: List[JudgeScoreBreakdown] = Field(default_factory=list)


class DependencyImpactResult(BaseModel):
    affected_functions: List[str] = Field(default_factory=list)
    affected_symbols: List[str] = Field(default_factory=list)
    impacted_files: List[str] = Field(default_factory=list)
    impact_size: int = 0


class TestPredictionResult(BaseModel):
    affected_tests: List[str] = Field(default_factory=list)
    predicted_failures: List[str] = Field(default_factory=list)
    confidence: float = 0.0


class SimulationResult(BaseModel):
    candidate_plan_id: str
    dependency_impact: DependencyImpactResult = Field(default_factory=DependencyImpactResult)
    test_prediction: TestPredictionResult = Field(default_factory=TestPredictionResult)
    risk_score: float = 0.0
    risk_reasons: List[str] = Field(default_factory=list)
    confidence: float = 0.0
    allow_execution: bool = True


class PlannerState(BaseModel):
    workflow_goal: str = ""
    graph_context: Dict[str, Any] = Field(default_factory=dict)
    memory_state: Dict[str, Any] = Field(default_factory=dict)
    prior_failures: List[str] = Field(default_factory=list)
    tool_hints: Dict[str, float] = Field(default_factory=dict)


class PlannerAction(BaseModel):
    action_id: str
    action_type: str
    payload: Dict[str, Any] = Field(default_factory=dict)
    score: float = 0.0


class PlanBranch(BaseModel):
    branch_id: str
    actions: List[PlannerAction] = Field(default_factory=list)
    score: float = 0.0
    metadata: Dict[str, Any] = Field(default_factory=dict)


class BranchMemoryRecord(BaseModel):
    branch_id: str
    status: str
    score: float = 0.0
    rejection_reason: str = ""
    metadata: Dict[str, Any] = Field(default_factory=dict)


class BranchExecutionResult(BaseModel):
    branch_id: str
    tests_passed: int = 0
    tests_failed: int = 0
    runtime_cost_s: float = 0.0
    regression_count: int = 0
    metadata: Dict[str, Any] = Field(default_factory=dict)


class BranchComparisonResult(BaseModel):
    selected_branch_id: str = ""
    rankings: List[Dict[str, Any]] = Field(default_factory=list)
    summary: Dict[str, Any] = Field(default_factory=dict)


class PlannerDecision(BaseModel):
    selected_branch_id: str = ""
    branches: List[PlanBranch] = Field(default_factory=list)
    rationale: Dict[str, Any] = Field(default_factory=dict)
