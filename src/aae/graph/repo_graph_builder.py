from __future__ import annotations

from pathlib import Path
from typing import List

from aae.contracts.graph import GraphBuildResult, GraphEdge, GraphEdgeType, GraphSnapshot
from aae.graph.alias_resolver import AliasResolver
from aae.graph.ast_parser import ParsedPythonFile, PythonAstParser
from aae.graph.call_graph_builder import CallGraphBuilder
from aae.graph.coverage_mapper import CoverageMapper
from aae.graph.dataflow_builder import DataflowBuilder
from aae.graph.graph_store import SQLiteGraphStore
from aae.graph.inheritance_builder import InheritanceBuilder
from aae.graph.symbol_table import SymbolTableBuilder


class RepoGraphBuilder:
    def __init__(
        self,
        parser: PythonAstParser | None = None,
        symbol_table_builder: SymbolTableBuilder | None = None,
        alias_resolver: AliasResolver | None = None,
        inheritance_builder: InheritanceBuilder | None = None,
        call_graph_builder: CallGraphBuilder | None = None,
        dataflow_builder: DataflowBuilder | None = None,
        coverage_mapper: CoverageMapper | None = None,
    ) -> None:
        self.parser = parser or PythonAstParser()
        self.symbol_table_builder = symbol_table_builder or SymbolTableBuilder()
        self.alias_resolver = alias_resolver or AliasResolver()
        self.inheritance_builder = inheritance_builder or InheritanceBuilder()
        self.call_graph_builder = call_graph_builder or CallGraphBuilder()
        self.dataflow_builder = dataflow_builder or DataflowBuilder()
        self.coverage_mapper = coverage_mapper or CoverageMapper()

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

        symbols, references, _ = self.symbol_table_builder.build(parsed_files)
        references = self.alias_resolver.resolve(parsed_files, symbols, references)

        call_edges = self.call_graph_builder.build_edges(parsed_files)
        edges.extend(call_edges)
        nodes, inheritance_edges = self.inheritance_builder.build(parsed_files, symbols, nodes)
        edges.extend(inheritance_edges)
        nodes, dataflow_edges = self.dataflow_builder.build(parsed_files, nodes)
        edges.extend(dataflow_edges)
        coverage = self.coverage_mapper.build(str(root), parsed_files, symbols)
        edges.extend(
            [
                GraphEdge(
                    source_id=item.test_node_id,
                    target_id=item.target_symbol_id,
                    edge_type=GraphEdgeType.TESTS,
                    metadata={"source": item.source, "confidence": item.confidence, **item.metadata},
                )
                for item in coverage
                if item.target_symbol_id
            ]
        )

        snapshot = GraphSnapshot(
            root_path=str(root),
            nodes=nodes,
            edges=edges,
            symbols=symbols,
            references=references,
            coverage=coverage,
        )
        store = SQLiteGraphStore(sqlite_path=sqlite_path, json_path=json_path)
        store.save(snapshot)

        stats = {
            "file_count": len({node.path for node in nodes if node.path}),
            "function_count": len([node for node in nodes if node.node_type.value in {"function", "test"}]),
            "class_count": len([node for node in nodes if node.node_type.value == "class"]),
            "edge_count": len(edges),
            "symbol_count": len(symbols),
            "reference_count": len(references),
            "coverage_count": len(coverage),
        }
        return GraphBuildResult(
            snapshot=snapshot,
            root_path=str(root),
            sqlite_path=sqlite_path,
            json_path=json_path,
            stats=stats,
        )
