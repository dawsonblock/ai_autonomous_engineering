from __future__ import annotations

import json
import sqlite3
from abc import ABC, abstractmethod
from pathlib import Path

from aae.contracts.graph import CoverageAssociation, GraphEdge, GraphNode, GraphSnapshot, SemanticSummary, SymbolDefinition, SymbolReference


class GraphStore(ABC):
    @abstractmethod
    def save(self, snapshot: GraphSnapshot) -> None:
        raise NotImplementedError

    @abstractmethod
    def load(self) -> GraphSnapshot:
        raise NotImplementedError


class SQLiteGraphStore(GraphStore):
    def __init__(self, sqlite_path: str, json_path: str | None = None) -> None:
        self.sqlite_path = Path(sqlite_path)
        self.json_path = Path(json_path) if json_path else None

    def save(self, snapshot: GraphSnapshot) -> None:
        self.sqlite_path.parent.mkdir(parents=True, exist_ok=True)
        connection = sqlite3.connect(self.sqlite_path)
        try:
            cursor = connection.cursor()
            cursor.executescript(
                """
                DROP TABLE IF EXISTS graph_meta;
                DROP TABLE IF EXISTS graph_nodes;
                DROP TABLE IF EXISTS graph_edges;
                DROP TABLE IF EXISTS graph_symbols;
                DROP TABLE IF EXISTS graph_symbol_refs;
                DROP TABLE IF EXISTS graph_coverage;
                DROP TABLE IF EXISTS graph_semantic_summaries;
                CREATE TABLE graph_meta (root_path TEXT, language TEXT);
                CREATE TABLE graph_nodes (
                    id TEXT PRIMARY KEY,
                    node_type TEXT,
                    name TEXT,
                    path TEXT,
                    qualname TEXT,
                    line INTEGER,
                    metadata TEXT
                );
                CREATE TABLE graph_edges (
                    source_id TEXT,
                    target_id TEXT,
                    edge_type TEXT,
                    metadata TEXT
                );
                CREATE TABLE graph_symbols (
                    symbol_id TEXT PRIMARY KEY,
                    name TEXT,
                    qualname TEXT,
                    symbol_type TEXT,
                    file_path TEXT,
                    line INTEGER,
                    class_scope TEXT,
                    signature TEXT,
                    metadata TEXT
                );
                CREATE TABLE graph_symbol_refs (
                    source_symbol_id TEXT,
                    referenced_name TEXT,
                    resolved_symbol_id TEXT,
                    file_path TEXT,
                    line INTEGER,
                    reference_type TEXT,
                    metadata TEXT
                );
                CREATE TABLE graph_coverage (
                    test_node_id TEXT,
                    target_symbol_id TEXT,
                    target_path TEXT,
                    source TEXT,
                    confidence REAL,
                    metadata TEXT
                );
                CREATE TABLE graph_semantic_summaries (
                    symbol_id TEXT,
                    cfg_nodes INTEGER,
                    branch_points INTEGER,
                    inferred_types TEXT,
                    signature TEXT,
                    resolved_calls TEXT,
                    metadata TEXT
                );
                """
            )
            cursor.execute(
                "INSERT INTO graph_meta(root_path, language) VALUES(?, ?)",
                (snapshot.root_path, snapshot.language),
            )
            cursor.executemany(
                """
                INSERT INTO graph_nodes(id, node_type, name, path, qualname, line, metadata)
                VALUES(?, ?, ?, ?, ?, ?, ?)
                """,
                [
                    (
                        node.id,
                        node.node_type.value,
                        node.name,
                        node.path,
                        node.qualname,
                        node.line,
                        json.dumps(node.metadata, sort_keys=True),
                    )
                    for node in snapshot.nodes
                ],
            )
            cursor.executemany(
                """
                INSERT INTO graph_edges(source_id, target_id, edge_type, metadata)
                VALUES(?, ?, ?, ?)
                """,
                [
                    (
                        edge.source_id,
                        edge.target_id,
                        edge.edge_type.value,
                        json.dumps(edge.metadata, sort_keys=True),
                    )
                    for edge in snapshot.edges
                ],
            )
            cursor.executemany(
                """
                INSERT INTO graph_symbols(symbol_id, name, qualname, symbol_type, file_path, line, class_scope, signature, metadata)
                VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                [
                    (
                        symbol.symbol_id,
                        symbol.name,
                        symbol.qualname,
                        symbol.symbol_type,
                        symbol.file_path,
                        symbol.line,
                        symbol.class_scope,
                        symbol.signature,
                        json.dumps(symbol.metadata, sort_keys=True),
                    )
                    for symbol in snapshot.symbols
                ],
            )
            cursor.executemany(
                """
                INSERT INTO graph_symbol_refs(source_symbol_id, referenced_name, resolved_symbol_id, file_path, line, reference_type, metadata)
                VALUES(?, ?, ?, ?, ?, ?, ?)
                """,
                [
                    (
                        reference.source_symbol_id,
                        reference.referenced_name,
                        reference.resolved_symbol_id,
                        reference.file_path,
                        reference.line,
                        reference.reference_type,
                        json.dumps(reference.metadata, sort_keys=True),
                    )
                    for reference in snapshot.references
                ],
            )
            cursor.executemany(
                """
                INSERT INTO graph_coverage(test_node_id, target_symbol_id, target_path, source, confidence, metadata)
                VALUES(?, ?, ?, ?, ?, ?)
                """,
                [
                    (
                        item.test_node_id,
                        item.target_symbol_id,
                        item.target_path,
                        item.source,
                        item.confidence,
                        json.dumps(item.metadata, sort_keys=True),
                    )
                    for item in snapshot.coverage
                ],
            )
            cursor.executemany(
                """
                INSERT INTO graph_semantic_summaries(symbol_id, cfg_nodes, branch_points, inferred_types, signature, resolved_calls, metadata)
                VALUES(?, ?, ?, ?, ?, ?, ?)
                """,
                [
                    (
                        summary.symbol_id,
                        summary.cfg_nodes,
                        summary.branch_points,
                        json.dumps(summary.inferred_types, sort_keys=True),
                        summary.signature,
                        json.dumps(summary.resolved_calls, sort_keys=True),
                        json.dumps(summary.metadata, sort_keys=True),
                    )
                    for summary in snapshot.semantic_summaries
                ],
            )
            connection.commit()
        finally:
            connection.close()
        if self.json_path is not None:
            self.json_path.parent.mkdir(parents=True, exist_ok=True)
            self.json_path.write_text(
                json.dumps(snapshot.model_dump(mode="json"), indent=2, sort_keys=True),
                encoding="utf-8",
            )

    def load(self) -> GraphSnapshot:
        connection = sqlite3.connect(self.sqlite_path)
        try:
            cursor = connection.cursor()
            meta = cursor.execute("SELECT root_path, language FROM graph_meta").fetchone()
            nodes = [
                GraphNode(
                    id=row[0],
                    node_type=row[1],
                    name=row[2],
                    path=row[3] or "",
                    qualname=row[4] or "",
                    line=row[5],
                    metadata=json.loads(row[6] or "{}"),
                )
                for row in cursor.execute(
                    "SELECT id, node_type, name, path, qualname, line, metadata FROM graph_nodes"
                )
            ]
            edges = [
                GraphEdge(
                    source_id=row[0],
                    target_id=row[1],
                    edge_type=row[2],
                    metadata=json.loads(row[3] or "{}"),
                )
                for row in cursor.execute(
                    "SELECT source_id, target_id, edge_type, metadata FROM graph_edges"
                )
            ]
            symbols = [
                SymbolDefinition(
                    symbol_id=row[0],
                    name=row[1],
                    qualname=row[2],
                    symbol_type=row[3],
                    file_path=row[4],
                    line=row[5],
                    class_scope=row[6] or "",
                    signature=row[7] or "",
                    metadata=json.loads(row[8] or "{}"),
                )
                for row in cursor.execute(
                    "SELECT symbol_id, name, qualname, symbol_type, file_path, line, class_scope, signature, metadata FROM graph_symbols"
                )
            ]
            references = [
                SymbolReference(
                    source_symbol_id=row[0] or "",
                    referenced_name=row[1],
                    resolved_symbol_id=row[2] or "",
                    file_path=row[3],
                    line=row[4],
                    reference_type=row[5] or "",
                    metadata=json.loads(row[6] or "{}"),
                )
                for row in cursor.execute(
                    "SELECT source_symbol_id, referenced_name, resolved_symbol_id, file_path, line, reference_type, metadata FROM graph_symbol_refs"
                )
            ]
            coverage = [
                CoverageAssociation(
                    test_node_id=row[0],
                    target_symbol_id=row[1] or "",
                    target_path=row[2] or "",
                    source=row[3] or "static",
                    confidence=float(row[4] or 0.0),
                    metadata=json.loads(row[5] or "{}"),
                )
                for row in cursor.execute(
                    "SELECT test_node_id, target_symbol_id, target_path, source, confidence, metadata FROM graph_coverage"
                )
            ]
            semantic_summaries = [
                SemanticSummary(
                    symbol_id=row[0],
                    cfg_nodes=int(row[1] or 0),
                    branch_points=int(row[2] or 0),
                    inferred_types=json.loads(row[3] or "{}"),
                    signature=row[4] or "",
                    resolved_calls=json.loads(row[5] or "[]"),
                    metadata=json.loads(row[6] or "{}"),
                )
                for row in cursor.execute(
                    "SELECT symbol_id, cfg_nodes, branch_points, inferred_types, signature, resolved_calls, metadata FROM graph_semantic_summaries"
                )
            ]
        finally:
            connection.close()
        return GraphSnapshot(
            root_path=meta[0],
            language=meta[1],
            nodes=nodes,
            edges=edges,
            symbols=symbols,
            references=references,
            coverage=coverage,
            semantic_summaries=semantic_summaries,
        )
