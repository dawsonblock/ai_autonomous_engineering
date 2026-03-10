from __future__ import annotations

from enum import Enum
from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field


class GraphNodeType(str, Enum):
    FILE = "file"
    CLASS = "class"
    FUNCTION = "function"
    MODULE = "module"
    TEST = "test"
    EXTERNAL = "external"


class GraphEdgeType(str, Enum):
    CALLS = "CALLS"
    IMPORTS = "IMPORTS"
    DEFINES = "DEFINES"
    TESTS = "TESTS"
    INHERITS = "INHERITS"
    OVERRIDES = "OVERRIDES"
    IMPLEMENTS = "IMPLEMENTS"
    READS = "READS"
    WRITES = "WRITES"
    PARAM_FLOW = "PARAM_FLOW"


class GraphNode(BaseModel):
    id: str
    node_type: GraphNodeType
    name: str
    path: str = ""
    qualname: str = ""
    line: Optional[int] = None
    metadata: Dict[str, Any] = Field(default_factory=dict)


class GraphEdge(BaseModel):
    source_id: str
    target_id: str
    edge_type: GraphEdgeType
    metadata: Dict[str, Any] = Field(default_factory=dict)


class SymbolDefinition(BaseModel):
    symbol_id: str
    name: str
    qualname: str
    symbol_type: str
    file_path: str
    line: Optional[int] = None
    class_scope: str = ""
    signature: str = ""
    metadata: Dict[str, Any] = Field(default_factory=dict)


class SymbolReference(BaseModel):
    source_symbol_id: str = ""
    referenced_name: str
    resolved_symbol_id: str = ""
    file_path: str
    line: Optional[int] = None
    reference_type: str = ""
    metadata: Dict[str, Any] = Field(default_factory=dict)


class CoverageAssociation(BaseModel):
    test_node_id: str
    target_symbol_id: str = ""
    target_path: str = ""
    source: str = "static"
    confidence: float = 0.0
    metadata: Dict[str, Any] = Field(default_factory=dict)


class SemanticSummary(BaseModel):
    symbol_id: str
    cfg_nodes: int = 0
    branch_points: int = 0
    inferred_types: Dict[str, str] = Field(default_factory=dict)
    signature: str = ""
    resolved_calls: List[str] = Field(default_factory=list)
    metadata: Dict[str, Any] = Field(default_factory=dict)


class GraphSnapshot(BaseModel):
    root_path: str
    language: str = "python-first"
    nodes: List[GraphNode] = Field(default_factory=list)
    edges: List[GraphEdge] = Field(default_factory=list)
    symbols: List[SymbolDefinition] = Field(default_factory=list)
    references: List[SymbolReference] = Field(default_factory=list)
    coverage: List[CoverageAssociation] = Field(default_factory=list)
    semantic_summaries: List[SemanticSummary] = Field(default_factory=list)


class GraphBuildResult(BaseModel):
    snapshot: GraphSnapshot
    root_path: str
    sqlite_path: str
    json_path: str
    stats: Dict[str, Any] = Field(default_factory=dict)


class GraphQueryRequest(BaseModel):
    query_name: str
    symbol: Optional[str] = None
    module: Optional[str] = None
    max_depth: int = 5


class GraphQueryResult(BaseModel):
    query_name: str
    items: List[Dict[str, Any]] = Field(default_factory=list)
    paths: List[List[str]] = Field(default_factory=list)
    summary: Dict[str, Any] = Field(default_factory=dict)


class RepoWorkspace(BaseModel):
    workflow_id: str
    source: str
    repo_path: str
    artifacts_dir: str
    checkout_ref: Optional[str] = None
    graph_sqlite_path: Optional[str] = None
    graph_json_path: Optional[str] = None
    metadata: Dict[str, Any] = Field(default_factory=dict)
