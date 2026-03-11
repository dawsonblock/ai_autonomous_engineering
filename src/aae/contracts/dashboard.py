from __future__ import annotations

from datetime import datetime, timezone
from typing import Any, Dict, List, Literal

from pydantic import BaseModel, Field, model_validator


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


WorkflowType = Literal["research_only", "security_only", "swe_only", "secure_build"]


class DashboardWorkflowLaunchRequest(BaseModel):
    workflow: WorkflowType = "secure_build"
    query: str = ""
    goal: str = ""
    repo_url: str = ""
    include_research: bool = False
    include_post_audit: bool = False
    workflow_id: str = ""

    @model_validator(mode="after")
    def validate_request(self) -> "DashboardWorkflowLaunchRequest":
        if self.workflow == "research_only" and not self.query.strip():
            raise ValueError("query is required for research_only")
        if self.workflow == "security_only" and not self.repo_url.strip():
            raise ValueError("repo_url is required for security_only")
        if self.workflow in {"swe_only", "secure_build"}:
            if not self.goal.strip():
                raise ValueError("goal is required for swe workflows")
            if not self.repo_url.strip():
                raise ValueError("repo_url is required for swe workflows")
        if self.workflow == "secure_build" and self.include_research and not self.query.strip():
            raise ValueError("query is required when include_research is enabled")
        return self


class DashboardWorkflowSummary(BaseModel):
    workflow_id: str
    workflow_type: str
    status: str = "pending"
    started_at: datetime | None = None
    updated_at: datetime | None = None
    completed_at: datetime | None = None
    metadata: Dict[str, Any] = Field(default_factory=dict)
    final_states: Dict[str, str] = Field(default_factory=dict)
    event_count: int = 0
    active_tasks: List[str] = Field(default_factory=list)
    trust_levels: List[str] = Field(default_factory=list)


class DashboardWorkflowDetail(BaseModel):
    summary: DashboardWorkflowSummary
    launch_request: Dict[str, Any] = Field(default_factory=dict)
    events: List[Dict[str, Any]] = Field(default_factory=list)
    memory_snapshot: Dict[str, Any] = Field(default_factory=dict)
    artifacts: Dict[str, Any] = Field(default_factory=dict)
    planner: Dict[str, Any] = Field(default_factory=dict)


class DashboardBenchmarkRunRequest(BaseModel):
    corpus_path: str = ""


class DashboardBenchmarkSummary(BaseModel):
    run_id: str = ""
    metrics: Dict[str, Any] = Field(default_factory=dict)
    report_path: str = ""
    markdown_report_path: str = ""
    generated_at: datetime = Field(default_factory=utc_now)


class SystemDiagnostic(BaseModel):
    name: str
    status: str
    summary: str
    details: Dict[str, Any] = Field(default_factory=dict)


class RuntimeOverrideProfile(BaseModel):
    controller_concurrency: int | None = None
    planner: Dict[str, Any] = Field(default_factory=dict)
    localization: Dict[str, float] = Field(default_factory=dict)
    ui: Dict[str, Any] = Field(default_factory=dict)
    updated_at: datetime = Field(default_factory=utc_now)
