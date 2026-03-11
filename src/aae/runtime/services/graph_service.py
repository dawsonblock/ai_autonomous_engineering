from __future__ import annotations

from pathlib import Path

from aae.code_analysis.context_ranker import ContextRanker
from aae.graph.graph_query import GraphQueryEngine
from aae.graph.repo_graph_builder import RepoGraphBuilder
from aae.memory.graph_memory import GraphMemory
from aae.persistence.graph_store import PostgresGraphStore
from aae.tools.graph_tools import GraphContextBuilder


class GraphService:
    def __init__(
        self,
        artifacts_dir: str,
        graph_builder: RepoGraphBuilder | None = None,
        graph_memory: GraphMemory | None = None,
        persistent_graph_store: PostgresGraphStore | None = None,
        context_ranker: ContextRanker | None = None,
    ) -> None:
        self.artifacts_dir = artifacts_dir
        self.graph_builder = graph_builder or RepoGraphBuilder()
        self.graph_memory = graph_memory or GraphMemory(base_dir=str(Path(artifacts_dir) / "memory" / "graphs"))
        self.persistent_graph_store = persistent_graph_store or PostgresGraphStore()
        self.context_ranker = context_ranker or ContextRanker()

    def ensure_graph(self, workflow_id: str, repo_path: str, graph_build: dict | None = None):
        if graph_build is None:
            graph_dir = Path(self.artifacts_dir) / "graphs" / workflow_id
            graph_dir.mkdir(parents=True, exist_ok=True)
            build_result = self.graph_builder.build(
                repo_path=repo_path,
                sqlite_path=str(graph_dir / "repo_graph.sqlite3"),
                json_path=str(graph_dir / "repo_graph.json"),
            )
            graph_build = build_result.model_dump(mode="json")
            self.graph_memory.store(workflow_id, build_result)
            self.persistent_graph_store.store_build_result(workflow_id, build_result)
        graph = GraphQueryEngine.from_sqlite(graph_build["sqlite_path"])
        return graph_build, graph

    def build_context(self, goal: str, graph: GraphQueryEngine, behavior_context: dict | None = None, failure_evidence: list[dict] | None = None) -> dict:
        return GraphContextBuilder(graph, context_ranker=self.context_ranker).build(
            goal,
            behavior_context=behavior_context or {},
            failure_evidence=failure_evidence or [],
        )
