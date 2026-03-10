from __future__ import annotations

from pathlib import Path
from typing import List

from aae.contracts.graph import GraphBuildResult, GraphSnapshot
from aae.graph.ast_parser import ParsedPythonFile, PythonAstParser
from aae.graph.call_graph_builder import CallGraphBuilder
from aae.graph.graph_store import SQLiteGraphStore


class RepoGraphBuilder:
    def __init__(self, parser: PythonAstParser | None = None, call_graph_builder: CallGraphBuilder | None = None) -> None:
        self.parser = parser or PythonAstParser()
        self.call_graph_builder = call_graph_builder or CallGraphBuilder()

    def build(self, repo_path: str, sqlite_path: str, json_path: str) -> GraphBuildResult:
        root = Path(repo_path).resolve()
        parsed_files: List[ParsedPythonFile] = []
        nodes = []
        edges = []

        for file_path in sorted(root.rglob("*")):
            if not file_path.is_file():
                continue
            if file_path.name.startswith("."):
                continue
            if file_path.suffix != ".py":
                relative = file_path.relative_to(root).as_posix()
                continue
            parsed = self.parser.parse_file(str(root), str(file_path))
            parsed_files.append(parsed)
            nodes.extend(parsed.nodes)
            edges.extend(parsed.edges)

        edges.extend(self.call_graph_builder.build_edges(parsed_files))
        snapshot = GraphSnapshot(root_path=str(root), nodes=nodes, edges=edges)
        store = SQLiteGraphStore(sqlite_path=sqlite_path, json_path=json_path)
        store.save(snapshot)

        stats = {
            "file_count": len({node.path for node in nodes if node.path}),
            "function_count": len([node for node in nodes if node.node_type.value in {"function", "test"}]),
            "class_count": len([node for node in nodes if node.node_type.value == "class"]),
            "edge_count": len(edges),
        }
        return GraphBuildResult(
            snapshot=snapshot,
            root_path=str(root),
            sqlite_path=sqlite_path,
            json_path=json_path,
            stats=stats,
        )
