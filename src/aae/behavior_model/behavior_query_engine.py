from __future__ import annotations

from collections import Counter, defaultdict, deque
from typing import Dict, Iterable, List

from aae.contracts.behavior import BehaviorEdgeType, BehaviorQueryResult, BehaviorSnapshot


class BehaviorQueryEngine:
    def __init__(self, snapshot: BehaviorSnapshot) -> None:
        self.snapshot = snapshot
        self.nodes = {node.id: node for node in snapshot.nodes}
        self.outgoing = defaultdict(list)
        self.incoming = defaultdict(list)
        for edge in snapshot.edges:
            self.outgoing[edge.source_id].append(edge)
            self.incoming[edge.target_id].append(edge)

    def functions_for_goal(self, symbols: Iterable[str]) -> BehaviorQueryResult:
        lowered = {symbol.lower() for symbol in symbols if symbol}
        matches = []
        for node in self.snapshot.nodes:
            if node.node_type.value != "function":
                continue
            score = 0.0
            for symbol in lowered:
                if symbol in node.name.lower():
                    score += 0.7
                if symbol in node.qualname.lower():
                    score += 0.2
            if score:
                matches.append({"id": node.id, "qualname": node.qualname, "path": node.path, "score": round(score, 3)})
        matches.sort(key=lambda item: item["score"], reverse=True)
        return BehaviorQueryResult(query_name="functions_for_goal", items=matches[:8], summary={"match_count": len(matches)})

    def trace_overlap(self, symbols: Iterable[str]) -> BehaviorQueryResult:
        lowered = {symbol.lower() for symbol in symbols if symbol}
        counts = Counter()
        for trace in self.snapshot.traces:
            if any(symbol in trace.function.lower() for symbol in lowered):
                counts[(trace.file_path, trace.function)] += 1
        items = [
            {"file_path": file_path, "function": function, "trace_hits": count}
            for (file_path, function), count in counts.most_common()
        ]
        return BehaviorQueryResult(query_name="trace_overlap", items=items, summary={"match_count": len(items)})

    def suspicious_files(self, symbols: Iterable[str]) -> BehaviorQueryResult:
        function_matches = self.functions_for_goal(symbols).items
        trace_matches = self.trace_overlap(symbols).items
        scores: Dict[str, float] = defaultdict(float)
        for match in function_matches:
            scores[match["path"]] += float(match["score"])
        for match in trace_matches:
            scores[match["file_path"]] += min(1.0, float(match["trace_hits"]) * 0.1)
        items = [{"path": path, "score": round(score, 3)} for path, score in sorted(scores.items(), key=lambda item: item[1], reverse=True)]
        return BehaviorQueryResult(query_name="suspicious_files", items=items, summary={"file_count": len(items)})

    def causal_path(self, symbol: str, max_depth: int = 4) -> BehaviorQueryResult:
        start_nodes = [node for node in self.snapshot.nodes if node.node_type.value == "function" and symbol.lower() in node.qualname.lower()]
        paths: List[List[str]] = []
        for node in start_nodes[:3]:
            queue = deque([(node.id, [node.qualname], 0)])
            while queue:
                current_id, path, depth = queue.popleft()
                if depth >= max_depth:
                    paths.append(path)
                    continue
                next_edges = [edge for edge in self.outgoing[current_id] if edge.edge_type in {BehaviorEdgeType.CALLS, BehaviorEdgeType.PRODUCES, BehaviorEdgeType.MODIFIES}]
                if not next_edges:
                    paths.append(path)
                    continue
                for edge in next_edges:
                    target = self.nodes.get(edge.target_id)
                    if target is None or target.qualname in path:
                        continue
                    queue.append((target.id, path + [target.qualname], depth + 1))
        return BehaviorQueryResult(query_name="causal_path", items=[{"path": path} for path in paths], summary={"path_count": len(paths), "symbol": symbol})
