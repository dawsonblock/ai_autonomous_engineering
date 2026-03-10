from __future__ import annotations

import json
from typing import Any

from aae.persistence.db import PostgresDatabase


EVALUATIONS_DDL = """
CREATE TABLE IF NOT EXISTS aae_evaluations (
    run_id TEXT NOT NULL,
    case_id TEXT NOT NULL,
    success BOOLEAN NOT NULL,
    runtime FLOAT NOT NULL DEFAULT 0,
    patch_size INT NOT NULL DEFAULT 0,
    regression_count INT NOT NULL DEFAULT 0,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (run_id, case_id)
)
"""


class PostgresEvaluationStore:
    def __init__(self, database: PostgresDatabase | None = None) -> None:
        self.database = database or PostgresDatabase()

    @property
    def enabled(self) -> bool:
        return self.database.enabled

    def record_case(self, run_id: str, case_id: str, payload: dict[str, Any]) -> None:
        if not self.enabled:
            return
        self.database.execute_ddl(EVALUATIONS_DDL)
        with self.database.connection() as connection:
            with connection.cursor() as cursor:
                cursor.execute(
                    """
                    INSERT INTO aae_evaluations(run_id, case_id, success, runtime, patch_size, regression_count, payload)
                    VALUES (%s, %s, %s, %s, %s, %s, %s::jsonb)
                    ON CONFLICT (run_id, case_id) DO UPDATE SET
                        success = EXCLUDED.success,
                        runtime = EXCLUDED.runtime,
                        patch_size = EXCLUDED.patch_size,
                        regression_count = EXCLUDED.regression_count,
                        payload = EXCLUDED.payload
                    """,
                    (
                        run_id,
                        case_id,
                        bool(payload.get("fixed", False)),
                        float(payload.get("runtime_cost_s", 0.0) or 0.0),
                        int(payload.get("patch_size", 0) or 0),
                        int(payload.get("regression_count", 0) or 0),
                        json.dumps(payload, sort_keys=True),
                    ),
                )
            connection.commit()

    def record_summary(self, run_id: str, payload: dict[str, Any]) -> None:
        self.record_case(
            run_id=run_id,
            case_id="__summary__",
            payload={
                "fixed": bool(payload.get("regression_summary", {}).get("passed", False)),
                "runtime_cost_s": float(payload.get("metrics", {}).get("runtime_per_success_s", 0.0) or 0.0),
                "patch_size": int(payload.get("metrics", {}).get("median_patch_size", 0) or 0),
                "regression_count": int(round(float(payload.get("metrics", {}).get("regression_rate", 0.0) or 0.0) * 1000)),
                **payload,
            },
        )
