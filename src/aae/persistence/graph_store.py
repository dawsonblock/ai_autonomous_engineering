from __future__ import annotations

import json
from typing import Any

from aae.contracts.graph import GraphBuildResult
from aae.persistence.db import PostgresDatabase


GRAPH_RUNS_DDL = """
CREATE TABLE IF NOT EXISTS aae_graph_runs (
    workflow_id TEXT PRIMARY KEY,
    root_path TEXT NOT NULL,
    sqlite_path TEXT NOT NULL,
    json_path TEXT NOT NULL,
    stats JSONB NOT NULL,
    snapshot JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
"""


class PostgresGraphStore:
    def __init__(self, database: PostgresDatabase | None = None) -> None:
        self.database = database or PostgresDatabase()

    @property
    def enabled(self) -> bool:
        return self.database.enabled

    def store_build_result(self, workflow_id: str, build_result: GraphBuildResult) -> None:
        if not self.enabled:
            return
        self.database.execute_ddl(GRAPH_RUNS_DDL)
        payload = build_result.model_dump(mode="json")
        with self.database.connection() as connection:
            with connection.cursor() as cursor:
                cursor.execute(
                    """
                    INSERT INTO aae_graph_runs(workflow_id, root_path, sqlite_path, json_path, stats, snapshot)
                    VALUES (%s, %s, %s, %s, %s::jsonb, %s::jsonb)
                    ON CONFLICT (workflow_id) DO UPDATE SET
                        root_path = EXCLUDED.root_path,
                        sqlite_path = EXCLUDED.sqlite_path,
                        json_path = EXCLUDED.json_path,
                        stats = EXCLUDED.stats,
                        snapshot = EXCLUDED.snapshot
                    """,
                    (
                        workflow_id,
                        build_result.root_path,
                        build_result.sqlite_path,
                        build_result.json_path,
                        json.dumps(build_result.stats, sort_keys=True),
                        json.dumps(payload["snapshot"], sort_keys=True),
                    ),
                )
            connection.commit()

    def load(self, workflow_id: str) -> dict[str, Any] | None:
        if not self.enabled:
            return None
        self.database.execute_ddl(GRAPH_RUNS_DDL)
        with self.database.connection() as connection:
            with connection.cursor() as cursor:
                cursor.execute(
                    "SELECT root_path, sqlite_path, json_path, stats, snapshot FROM aae_graph_runs WHERE workflow_id = %s",
                    (workflow_id,),
                )
                row = cursor.fetchone()
        if row is None:
            return None
        return {
            "root_path": row[0],
            "sqlite_path": row[1],
            "json_path": row[2],
            "stats": row[3],
            "snapshot": row[4],
        }
