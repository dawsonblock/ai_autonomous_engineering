from __future__ import annotations

import os
from contextlib import contextmanager
from typing import Iterator


def db_enabled() -> bool:
    return bool(os.getenv("AAE_DATABASE_URL", "").strip())


class PostgresDatabase:
    def __init__(self, dsn: str | None = None) -> None:
        self.dsn = dsn or os.getenv("AAE_DATABASE_URL", "").strip()

    @property
    def enabled(self) -> bool:
        return bool(self.dsn)

    def ensure_available(self) -> None:
        if not self.enabled:
            raise RuntimeError("AAE_DATABASE_URL is not configured")
        try:
            import psycopg  # noqa: F401
        except ImportError as exc:  # pragma: no cover - depends on env
            raise RuntimeError("psycopg is required for PostgreSQL persistence") from exc

    @contextmanager
    def connection(self) -> Iterator[object]:
        self.ensure_available()
        import psycopg

        with psycopg.connect(self.dsn) as connection:
            yield connection

    def execute_ddl(self, ddl: str) -> None:
        if not self.enabled:
            return
        with self.connection() as connection:
            with connection.cursor() as cursor:
                cursor.execute(ddl)
            connection.commit()
