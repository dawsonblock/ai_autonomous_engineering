from __future__ import annotations

from typing import Any, Dict

from .models import LocalizationResult, RankedFile, RankedFunction, RankedSpan


class LocalizationSerializer:
    @classmethod
    def from_agent_payload(cls, payload: Dict[str, Any]) -> LocalizationResult:
        suspicious_locations = payload.get("suspicious_locations") or []
        top_files = payload.get("top_files") or payload.get("localized_files") or payload.get("candidate_files") or []
        top_functions = payload.get("top_functions") or []
        top_spans = payload.get("top_spans") or []

        ranked_files: list[RankedFile] = []
        seen_files: set[str] = set()
        for entry in top_files:
            if isinstance(entry, str):
                file_path = entry
                score = 0.0
                evidence_sources = []
                score_components = {}
                reasons = []
            else:
                file_path = entry.get("file_path") or entry.get("path") or ""
                score = float(entry.get("score") or entry.get("confidence") or 0.0)
                evidence_sources = list(entry.get("evidence_sources") or [])
                score_components = dict(entry.get("score_components") or {})
                reasons = list(entry.get("reasons") or ([entry.get("reason")] if entry.get("reason") else []))
            if not file_path or file_path in seen_files:
                continue
            seen_files.add(file_path)
            ranked_files.append(
                RankedFile(
                    file_path=file_path,
                    score=score,
                    reasons=reasons,
                    evidence_sources=evidence_sources,
                    score_components=score_components,
                )
            )

        ranked_functions: list[RankedFunction] = []
        seen_functions: set[tuple[str, str]] = set()
        for entry in top_functions:
            if isinstance(entry, str):
                if "::" in entry:
                    file_path, function_name = entry.split("::", 1)
                else:
                    file_path, function_name = "", entry
                score = 0.0
                evidence_sources = []
                score_components = {}
                reasons = []
            else:
                file_path = entry.get("file_path") or entry.get("path") or ""
                function_name = entry.get("function_name") or entry.get("symbol") or entry.get("name") or ""
                score = float(entry.get("score") or entry.get("confidence") or 0.0)
                evidence_sources = list(entry.get("evidence_sources") or [])
                score_components = dict(entry.get("score_components") or {})
                reasons = list(entry.get("reasons") or [])
            key = (file_path, function_name)
            if not function_name or key in seen_functions:
                continue
            seen_functions.add(key)
            ranked_functions.append(
                RankedFunction(
                    file_path=file_path,
                    function_name=function_name,
                    score=score,
                    reasons=reasons,
                    evidence_sources=evidence_sources,
                    score_components=score_components,
                )
            )

        ranked_spans: list[RankedSpan] = []
        for entry in top_spans:
            file_path = entry.get("file_path") or entry.get("path") or ""
            if not file_path:
                continue
            ranked_spans.append(
                RankedSpan(
                    file_path=file_path,
                    start_line=int(entry.get("start_line") or 0),
                    end_line=int(entry.get("end_line") or 0),
                    score=float(entry.get("score") or entry.get("confidence") or 0.0),
                    span_type=str(entry.get("span_type") or "localized_span"),
                    reasons=list(entry.get("reasons") or []),
                    evidence_sources=list(entry.get("evidence_sources") or []),
                    score_components=dict(entry.get("score_components") or {}),
                )
            )

        for location in suspicious_locations:
            file_path = location.get("file_path") or ""
            function_name = location.get("symbol") or ""
            if file_path and file_path not in seen_files:
                seen_files.add(file_path)
                ranked_files.append(
                    RankedFile(
                        file_path=file_path,
                        score=float(location.get("confidence") or 0.0),
                        evidence_sources=list(location.get("evidence_sources") or []),
                        score_components=dict(location.get("score_components") or {}),
                    )
                )
            if function_name:
                key = (file_path, function_name)
                if key not in seen_functions:
                    seen_functions.add(key)
                    ranked_functions.append(
                        RankedFunction(
                            file_path=file_path,
                            function_name=function_name,
                            score=float(location.get("confidence") or 0.0),
                            evidence_sources=list(location.get("evidence_sources") or []),
                            score_components=dict(location.get("score_components") or {}),
                        )
                    )
            if file_path and (location.get("start_line") or location.get("end_line")):
                ranked_spans.append(
                    RankedSpan(
                        file_path=file_path,
                        start_line=int(location.get("start_line") or 0),
                        end_line=int(location.get("end_line") or 0),
                        score=float(location.get("confidence") or 0.0),
                        span_type="suspicious_location",
                        evidence_sources=list(location.get("evidence_sources") or []),
                        score_components=dict(location.get("score_components") or {}),
                    )
                )

        return LocalizationResult(
            files=ranked_files,
            functions=ranked_functions,
            spans=ranked_spans,
            summary=dict(payload.get("summary") or {}),
        )

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
