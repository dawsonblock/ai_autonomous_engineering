from __future__ import annotations

from collections import defaultdict
from typing import Dict, Iterable, List

from aae.behavior_model.behavior_query_engine import BehaviorQueryEngine
from aae.contracts.localization import FailureEvidence, LocalizationResult, SuspiciousLocation
from aae.graph.graph_query import GraphQueryEngine


class SuspiciousnessRanker:
    def rank(
        self,
        graph: GraphQueryEngine,
        behavior: BehaviorQueryEngine | None,
        candidate_symbols: Iterable[str],
        evidence: List[FailureEvidence],
    ) -> LocalizationResult:
        scores: Dict[tuple[str, str, int, int], Dict[str, float]] = defaultdict(lambda: defaultdict(float))
        evidence_sources: Dict[tuple[str, str, int, int], set[str]] = defaultdict(set)

        for symbol in candidate_symbols:
            for match in graph.find_functions(symbol).items:
                key = (match["path"], match["name"], int(match.get("line") or 0), int(match.get("line") or 0))
                scores[key]["graph_proximity"] += 0.5
                evidence_sources[key].add("graph_proximity")

        for item in evidence:
            key = (item.file_path, item.symbol, item.line, item.line)
            scores[key][item.source] += item.weight
            evidence_sources[key].add(item.source)

        if behavior is not None:
            overlap = behavior.trace_overlap(candidate_symbols).items
            for item in overlap:
                key = (item["file_path"], item["function"], 0, 0)
                scores[key]["trace_overlap"] += min(0.9, float(item["trace_hits"]) * 0.15)
                evidence_sources[key].add("trace_overlap")

        suspicious = []
        for key, components in scores.items():
            total = sum(components.values())
            if total <= 0:
                continue
            file_path, symbol, start_line, end_line = key
            if not file_path:
                continue
            suspicious.append(
                SuspiciousLocation(
                    file_path=file_path,
                    symbol=symbol,
                    start_line=start_line,
                    end_line=end_line or start_line,
                    confidence=min(0.99, round(total, 3)),
                    evidence_sources=sorted(evidence_sources[key]),
                    score_components={name: round(value, 3) for name, value in components.items()},
                )
            )
        suspicious.sort(key=lambda item: item.confidence, reverse=True)
        return LocalizationResult(
            suspicious_locations=suspicious[:8],
            evidence=evidence,
            summary={"location_count": len(suspicious)},
        )
