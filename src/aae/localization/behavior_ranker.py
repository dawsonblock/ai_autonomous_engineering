from __future__ import annotations

from collections import defaultdict
from typing import Any, Dict, Iterable, Tuple

from aae.behavior_model.behavior_query_engine import BehaviorQueryEngine

from .models import FailureSignal


class BehaviorRanker:
    def rank_functions(
        self,
        failure_signals: list[FailureSignal],
        context: Dict[str, Any],
        candidate_symbols: Iterable[str] | None = None,
    ) -> Dict[Tuple[str, str], float]:
        behavior_engine = context.get("behavior_engine")
        if not isinstance(behavior_engine, BehaviorQueryEngine):
            return {}
        symbols = list(candidate_symbols or [])
        for signal in failure_signals:
            if signal.test_name and signal.test_name != "unknown":
                symbols.append(signal.test_name)
            if signal.file_path and signal.file_path.endswith(".py"):
                symbols.append(signal.file_path.rsplit("/", 1)[-1].replace(".py", ""))
        ranked: Dict[Tuple[str, str], float] = defaultdict(float)
        for symbol in [item for item in symbols if item]:
            for overlap in behavior_engine.trace_overlap([symbol]).items[:10]:
                path = overlap.get("file_path", "")
                function = overlap.get("function", "")
                if not path or not function:
                    continue
                ranked[(path, function)] = max(ranked[(path, function)], min(1.0, float(overlap.get("trace_hits", 0)) * 0.18))
            for path_info in behavior_engine.causal_path(symbol).items[:5]:
                for step in path_info.get("path", []):
                    function = str(step)
                    short = function.split(".")[-1]
                    for file_match in behavior_engine.suspicious_files([short]).items[:5]:
                        path = file_match.get("path", "")
                        if not path:
                            continue
                        ranked[(path, short)] = max(ranked[(path, short)], float(file_match.get("score", 0.0) or 0.0))
        return dict(ranked)

    def rank_files(
        self,
        failure_signals: list[FailureSignal],
        context: Dict[str, Any],
        candidate_symbols: Iterable[str] | None = None,
    ) -> Dict[str, float]:
        file_scores: Dict[str, float] = defaultdict(float)
        for (file_path, _function_name), score in self.rank_functions(failure_signals, context, candidate_symbols).items():
            file_scores[file_path] = max(file_scores[file_path], score)
        return dict(file_scores)
