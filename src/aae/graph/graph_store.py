from __future__ import annotations

import json
import sqlite3
from abc import ABC, abstractmethod
from pathlib import Path

from aae.contracts.graph import GraphEdge, GraphNode, GraphSnapshot


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
        finally:
            connection.close()
        return GraphSnapshot(root_path=meta[0], language=meta[1], nodes=nodes, edges=edges)
