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


class SandboxRunResult(BaseModel):
    container_id: str
    commands: List[str]
    exit_code: int
    stdout: str = ""
    stderr: str = ""
    artifact_paths: List[str] = Field(default_factory=list)
    coverage_path: str = ""
    patch_apply_status: str = ""
    transport: str = "local"
