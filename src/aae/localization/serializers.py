from __future__ import annotations

import json
from typing import Any, Dict

from .models import LocalizationResult


class LocalizationSerializer:
    def to_json(self, result: LocalizationResult) -> str:
        return result.model_dump_json()

    def to_summary(self, result: LocalizationResult) -> Dict[str, Any]:
        return {
            "top_files": [f.file_path for f in result.files[:3]],
            "top_functions": [f"{f.file_path}::{f.function_name}" for f in result.functions[:5]],
            "spans_count": len(result.spans),
        }

    def to_llm_context(self, result: LocalizationResult) -> str:
        lines = []
        for span in result.spans:
            lines.append(
                f"Span in {span.file_path} (Lines {span.start_line}-{span.end_line}) "
                f"type: {span.span_type} score: {span.score:.2f}"
            )
            if span.reasons:
                lines.append(f"  Reasons: {', '.join(span.reasons)}")
        return "\n".join(lines)
