from __future__ import annotations

import json
from typing import Any

from aae.contracts.graph import CoverageAssociation, SymbolDefinition, SymbolReference
from aae.persistence.db import PostgresDatabase


SYMBOLS_DDL = """
CREATE TABLE IF NOT EXISTS aae_symbols (
    symbol_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    qualname TEXT NOT NULL,
    symbol_type TEXT NOT NULL,
    file_path TEXT NOT NULL,
    line INT,
    class_scope TEXT NOT NULL DEFAULT '',
    signature TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
CREATE TABLE IF NOT EXISTS aae_symbol_refs (
    id BIGSERIAL PRIMARY KEY,
    source_symbol_id TEXT NOT NULL DEFAULT '',
    referenced_name TEXT NOT NULL,
    resolved_symbol_id TEXT NOT NULL DEFAULT '',
    file_path TEXT NOT NULL,
    line INT,
    reference_type TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
CREATE TABLE IF NOT EXISTS aae_symbol_coverage (
    id BIGSERIAL PRIMARY KEY,
    test_node_id TEXT NOT NULL,
    target_symbol_id TEXT NOT NULL DEFAULT '',
    target_path TEXT NOT NULL DEFAULT '',
    source TEXT NOT NULL DEFAULT 'static',
    confidence FLOAT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
)
"""


class PostgresSymbolStore:
    def __init__(self, database: PostgresDatabase | None = None) -> None:
        self.database = database or PostgresDatabase()

    @property
    def enabled(self) -> bool:
        return self.database.enabled

    def store_snapshot(
        self,
        symbols: list[SymbolDefinition],
        references: list[SymbolReference],
        coverage: list[CoverageAssociation],
    ) -> None:
        if not self.enabled:
            return
        self.database.execute_ddl(SYMBOLS_DDL)
        with self.database.connection() as connection:
            with connection.cursor() as cursor:
                cursor.executemany(
                    """
                    INSERT INTO aae_symbols(symbol_id, name, qualname, symbol_type, file_path, line, class_scope, signature, metadata)
                    VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s::jsonb)
                    ON CONFLICT (symbol_id) DO UPDATE SET
                        name = EXCLUDED.name,
                        qualname = EXCLUDED.qualname,
                        symbol_type = EXCLUDED.symbol_type,
                        file_path = EXCLUDED.file_path,
                        line = EXCLUDED.line,
                        class_scope = EXCLUDED.class_scope,
                        signature = EXCLUDED.signature,
                        metadata = EXCLUDED.metadata
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
                        for symbol in symbols
                    ],
                )
                cursor.execute("DELETE FROM aae_symbol_refs")
                cursor.execute("DELETE FROM aae_symbol_coverage")
                cursor.executemany(
                    """
                    INSERT INTO aae_symbol_refs(source_symbol_id, referenced_name, resolved_symbol_id, file_path, line, reference_type, metadata)
                    VALUES (%s, %s, %s, %s, %s, %s, %s::jsonb)
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
                        for reference in references
                    ],
                )
                cursor.executemany(
                    """
                    INSERT INTO aae_symbol_coverage(test_node_id, target_symbol_id, target_path, source, confidence, metadata)
                    VALUES (%s, %s, %s, %s, %s, %s::jsonb)
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
                        for item in coverage
                    ],
                )
            connection.commit()

    def find_references(self, symbol: str) -> list[dict[str, Any]]:
        if not self.enabled:
            return []
        self.database.execute_ddl(SYMBOLS_DDL)
        with self.database.connection() as connection:
            with connection.cursor() as cursor:
                cursor.execute(
                    """
                    SELECT referenced_name, resolved_symbol_id, file_path, line, reference_type, metadata
                    FROM aae_symbol_refs
                    WHERE referenced_name = %s OR resolved_symbol_id IN (
                        SELECT symbol_id FROM aae_symbols WHERE name = %s OR qualname = %s
                    )
                    ORDER BY file_path, line
                    """,
                    (symbol, symbol, symbol),
                )
                rows = cursor.fetchall()
        return [
            {
                "referenced_name": row[0],
                "resolved_symbol_id": row[1],
                "file_path": row[2],
                "line": row[3],
                "reference_type": row[4],
                "metadata": row[5],
            }
            for row in rows
        ]
