from __future__ import annotations

import math
import os
from typing import Any

import httpx

from aae.graph.graph_query import GraphQueryEngine


class ContextRanker:
    def __init__(self, embedding_model: str | None = None) -> None:
        self.embedding_model = embedding_model or os.getenv("AAE_CONTEXT_EMBEDDINGS_MODEL", "").strip()

    def rank(
        self,
        query: str,
        graph: GraphQueryEngine,
        graph_context: dict[str, Any],
        behavior_context: dict[str, Any] | None = None,
        failure_evidence: list[dict[str, Any]] | None = None,
    ) -> dict[str, Any]:
        failure_evidence = failure_evidence or []
        behavior_context = behavior_context or {}
        ranked_symbols = []
        ranked_files = []
        snippets = []

        suspicious_scores = {
            item.get("path", item.get("file_path", "")): float(item.get("score", 0.0))
            for item in behavior_context.get("suspicious_files", [])
        }
        evidence_scores = {}
        for item in failure_evidence:
            path = item.get("file_path", "")
            if not path:
                continue
            evidence_scores[path] = evidence_scores.get(path, 0.0) + float(item.get("weight", 0.0) or 0.0)

        for entry in graph_context.get("symbol_context", []):
            symbol = entry.get("symbol", "")
            matches = entry.get("matches", [])
            related = graph.rank_related_symbols(symbol).items
            references = graph.find_references(symbol).items
            lexical = _lexical_similarity(query, symbol)
            density = min(1.0, len(references) * 0.15)
            related_score = min(1.0, len(related) * 0.1)
            behavior_score = max((suspicious_scores.get(match.get("path", ""), 0.0) for match in matches), default=0.0)
            failure_score = max((evidence_scores.get(match.get("path", ""), 0.0) for match in matches), default=0.0)
            embedding_score = self._embedding_score(query, symbol)
            score = (lexical * 0.35) + (density * 0.2) + (related_score * 0.15) + (behavior_score * 0.2) + (failure_score * 0.1) + (embedding_score * 0.05)
            ranked_symbols.append(
                {
                    "symbol": symbol,
                    "score": round(score, 3),
                    "reference_count": len(references),
                    "related_symbols": related[:5],
                    "matches": matches[:3],
                }
            )
            for match in matches:
                file_path = match.get("path", "")
                if not file_path:
                    continue
                file_score = score + suspicious_scores.get(file_path, 0.0) + evidence_scores.get(file_path, 0.0)
                ranked_files.append({"path": file_path, "score": round(file_score, 3), "symbol": symbol})
                snippets.append(
                    {
                        "path": file_path,
                        "symbol": match.get("qualname", symbol),
                        "score": round(file_score, 3),
                        "reason": "ranked_context",
                    }
                )

        ranked_symbols.sort(key=lambda item: item["score"], reverse=True)
        ranked_files = _dedupe_ranked_files(ranked_files)
        snippets = _dedupe_ranked_snippets(snippets)
        return {
            "ranked_symbols": ranked_symbols[:6],
            "ranked_files": ranked_files[:8],
            "ranked_snippets": snippets[:12],
        }

    def _embedding_score(self, query: str, symbol: str) -> float:
        if not self.embedding_model or not os.getenv("OPENAI_API_KEY", "").strip():
            return 0.0
        try:
            with httpx.Client(timeout=10.0) as client:
                response = client.post(
                    "%s/embeddings" % os.getenv("OPENAI_BASE_URL", "https://api.openai.com/v1").rstrip("/"),
                    headers={
                        "Authorization": "Bearer %s" % os.getenv("OPENAI_API_KEY", "").strip(),
                        "Content-Type": "application/json",
                    },
                    json={"model": self.embedding_model, "input": [query, symbol]},
                )
                response.raise_for_status()
                data = response.json().get("data", [])
        except Exception:
            return 0.0
        if len(data) != 2:
            return 0.0
        left = data[0].get("embedding", [])
        right = data[1].get("embedding", [])
        if not left or not right:
            return 0.0
        numerator = sum(a * b for a, b in zip(left, right))
        left_norm = math.sqrt(sum(a * a for a in left))
        right_norm = math.sqrt(sum(b * b for b in right))
        if not left_norm or not right_norm:
            return 0.0
        return max(0.0, min(1.0, numerator / (left_norm * right_norm)))


def _lexical_similarity(left: str, right: str) -> float:
    left_tokens = {token.lower() for token in left.replace("_", " ").split() if token}
    right_tokens = {token.lower() for token in right.replace("_", " ").split() if token}
    if not left_tokens or not right_tokens:
        return 0.0
    overlap = len(left_tokens & right_tokens)
    return overlap / max(len(left_tokens), len(right_tokens))


def _dedupe_ranked_files(items: list[dict[str, Any]]) -> list[dict[str, Any]]:
    by_path = {}
    for item in items:
        existing = by_path.get(item["path"])
        if existing is None or item["score"] > existing["score"]:
            by_path[item["path"]] = item
    return sorted(by_path.values(), key=lambda item: item["score"], reverse=True)


def _dedupe_ranked_snippets(items: list[dict[str, Any]]) -> list[dict[str, Any]]:
    seen = set()
    ranked = []
    for item in sorted(items, key=lambda entry: entry["score"], reverse=True):
        key = (item["path"], item["symbol"])
        if key in seen:
            continue
        seen.add(key)
        ranked.append(item)
    return ranked
