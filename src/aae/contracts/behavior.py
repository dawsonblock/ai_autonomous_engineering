from __future__ import annotations

from enum import Enum
from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field


class BehaviorNodeType(str, Enum):
    FUNCTION = "function"
    VARIABLE = "variable"
    INPUT = "input"
    OUTPUT = "output"
    STATE = "state"


class BehaviorEdgeType(str, Enum):
    CALLS = "calls"
    MODIFIES = "modifies"
    PRODUCES = "produces"
    CONSUMES = "consumes"
    TRANSITIONS = "transitions"
    RETURNS = "returns"


class BehaviorNode(BaseModel):
    id: str
    node_type: BehaviorNodeType
    name: str
    path: str = ""
    qualname: str = ""
    line: Optional[int] = None
    metadata: Dict[str, Any] = Field(default_factory=dict)


class BehaviorEdge(BaseModel):
    source_id: str
    target_id: str
    edge_type: BehaviorEdgeType
    weight: float = 1.0
    metadata: Dict[str, Any] = Field(default_factory=dict)


class TraceRecord(BaseModel):
    event_type: str
    function: str = ""
    file_path: str = ""
    line: int = 0
    command_id: str = ""
    test_id: str = ""
    call_id: str = ""
    parent_call_id: str = ""
    args_summary: str = ""
    result_summary: str = ""
    exception_type: str = ""
    timestamp: str = ""
    metadata: Dict[str, Any] = Field(default_factory=dict)


class StateTransition(BaseModel):
    transition_id: str
    source_state: str
    target_state: str
    trigger: str = ""
    metadata: Dict[str, Any] = Field(default_factory=dict)


class BehaviorSnapshot(BaseModel):
    root_path: str
    nodes: List[BehaviorNode] = Field(default_factory=list)
    edges: List[BehaviorEdge] = Field(default_factory=list)
    traces: List[TraceRecord] = Field(default_factory=list)
    transitions: List[StateTransition] = Field(default_factory=list)
    metadata: Dict[str, Any] = Field(default_factory=dict)


class BehaviorQueryResult(BaseModel):
    query_name: str
    items: List[Dict[str, Any]] = Field(default_factory=list)
    summary: Dict[str, Any] = Field(default_factory=dict)
