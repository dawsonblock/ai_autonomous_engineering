from __future__ import annotations

from typing import Dict, List

from pydantic import BaseModel, Field


class SandboxRunSpec(BaseModel):
    repo_path: str
    commands: List[str]
    image: str = ""
    timeout_s: int = 300
    network_enabled: bool = False
    cpu_limit: str = "1.0"
    memory_limit: str = "512m"
    environment: Dict[str, str] = Field(default_factory=dict)
    artifact_dir: str = ""
    patch_diff: str = ""
    patch_bundle: List[str] = Field(default_factory=list)
    repair_constraints: List[str] = Field(default_factory=list)
    trace_enabled: bool = True
    install_dependencies: bool = False
    selected_tests: List[str] = Field(default_factory=list)
    command_id: str = ""
    tracing_root: str = ""
    ephemeral_test_paths: List[str] = Field(default_factory=list)


class SandboxRunResult(BaseModel):
    container_id: str
    commands: List[str]
    exit_code: int
    stdout: str = ""
    stderr: str = ""
    execution_mode: str = "local"
    trust_level: str = "degraded"
    fallback_reason: str = ""
    artifact_paths: List[str] = Field(default_factory=list)
    coverage_path: str = ""
    patch_apply_status: str = ""
    transport: str = "local"
    trace_paths: List[str] = Field(default_factory=list)
    test_output_paths: List[str] = Field(default_factory=list)
    dependency_install_status: str = ""
    applied_workspace: str = ""
    editable_workspace: str = ""
    rollback_status: str = ""
    counterexample_paths: List[str] = Field(default_factory=list)
    patch_apply_details: Dict[str, object] = Field(default_factory=dict)
