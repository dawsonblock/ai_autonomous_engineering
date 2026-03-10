from __future__ import annotations

import json
from typing import Any

from aae.persistence.db import PostgresDatabase


TRAJECTORIES_DDL = """
CREATE TABLE IF NOT EXISTS aae_trajectories (
    id BIGSERIAL PRIMARY KEY,
    namespace TEXT NOT NULL,
    record JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
"""


CHECKPOINTS_DDL = """
CREATE TABLE IF NOT EXISTS aae_checkpoints (
    id BIGSERIAL PRIMARY KEY,
    namespace TEXT NOT NULL,
    thread_id TEXT NOT NULL,
    parent_thread_id TEXT,
    state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(namespace, thread_id)
)
"""

class PostgresTrajectoryStore:
    def __init__(self, database: PostgresDatabase | None = None) -> None:
        self.database = database or PostgresDatabase()

    @property
    def enabled(self) -> bool:
        return self.database.enabled

    def append(self, namespace: str, record: dict[str, Any]) -> None:
        if not self.enabled:
            return
        self.database.execute_ddl(TRAJECTORIES_DDL)
        with self.database.connection() as connection:
            with connection.cursor() as cursor:
                cursor.execute(
                    "INSERT INTO aae_trajectories(namespace, record) VALUES (%s, %s::jsonb)",
                    (namespace, json.dumps(record, sort_keys=True)),
                )
            connection.commit()

    def read(self, namespace: str) -> list[dict[str, Any]]:
        if not self.enabled:
            return []
        self.database.execute_ddl(TRAJECTORIES_DDL)
        with self.database.connection() as connection:
            with connection.cursor() as cursor:
                cursor.execute(
                    "SELECT record FROM aae_trajectories WHERE namespace = %s ORDER BY id ASC",
                    (namespace,),
                )
                rows = cursor.fetchall()
        return [row[0] for row in rows]
        
    def save_checkpoint(self, namespace: str, thread_id: str, state: dict[str, Any], parent_thread_id: str | None = None) -> None:
        if not self.enabled:
            return
        self.database.execute_ddl(CHECKPOINTS_DDL)
        with self.database.connection() as connection:
            with connection.cursor() as cursor:
                cursor.execute(
                    """
                    INSERT INTO aae_checkpoints(namespace, thread_id, parent_thread_id, state)
                    VALUES (%s, %s, %s, %s::jsonb)
                    ON CONFLICT (namespace, thread_id) DO UPDATE SET state = EXCLUDED.state
                    """,
                    (namespace, thread_id, parent_thread_id, json.dumps(state, sort_keys=True)),
                )
            connection.commit()
            
    def get_checkpoint(self, namespace: str, thread_id: str) -> dict[str, Any] | None:
        if not self.enabled:
            return None
        self.database.execute_ddl(CHECKPOINTS_DDL)
        with self.database.connection() as connection:
            with connection.cursor() as cursor:
                cursor.execute(
                    "SELECT state FROM aae_checkpoints WHERE namespace = %s AND thread_id = %s",
                    (namespace, thread_id)
                )
                row = cursor.fetchone()
        return row[0] if (row and len(row) > 0) else None
