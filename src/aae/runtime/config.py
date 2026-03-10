from __future__ import annotations

import os
from pathlib import Path
from typing import Dict

import yaml
from pydantic import BaseModel, Field


class AgentFieldConfig(BaseModel):
    base_url: str
    api_key_env: str = "AGENTFIELD_API_KEY"
    poll_interval_s: float = 1.0
    request_timeout_s: float = 30.0


class ControllerConfig(BaseModel):
    max_concurrency: int = 4
    artifacts_dir: str = ".artifacts"


class SiblingConfig(BaseModel):
    path: str
    node_target: str


class SystemConfig(BaseModel):
    agentfield: AgentFieldConfig
    controller: ControllerConfig = Field(default_factory=ControllerConfig)
    siblings: Dict[str, SiblingConfig]

    @classmethod
    def load(cls, path: str) -> "SystemConfig":
        with Path(path).open("r", encoding="utf-8") as handle:
            data = yaml.safe_load(handle)
        return cls.model_validate(data)

    def api_key(self) -> str | None:
        return os.getenv(self.agentfield.api_key_env)
