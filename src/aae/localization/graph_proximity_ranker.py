from __future__ import annotations

from typing import Any, Dict, List, Tuple

from .models import FailureSignal


class GraphProximityRanker:
    def _get_seeds(self, failure_signals: List[FailureSignal], context: Dict[str, Any]) -> set[str]:
        seeds = set()
        
        for f in failure_signals:
            if f.test_name and f.test_name != "unknown":
                seeds.add(f.test_name)
                
        spectrum_funcs = context.get("spectrum_functions", {})
        if spectrum_funcs:
            top_funcs = sorted(spectrum_funcs.items(), key=lambda x: x[1], reverse=True)[:5]
            for (file_path, func_name), score in top_funcs:
                seeds.add(func_name)
                
        frames = context.get("stacktrace_frames", [])
        for frame in frames:
            if getattr(frame, "resolved_symbol_id", None):
                seeds.add(frame.resolved_symbol_id)
            elif frame.function_name:
                seeds.add(frame.function_name)
                
        return seeds

    def rank(self, failure_signals: List[FailureSignal], context: Dict[str, Any]) -> Dict[Any, float]:
        return self.rank_functions(failure_signals, context)

    def rank_functions(self, failure_signals: List[FailureSignal], context: Dict[str, Any]) -> Dict[Tuple[str, str], float]:
        graph_engine = context.get("graph_engine")
        if not graph_engine:
            return {}
            
        seeds = self._get_seeds(failure_signals, context)
        func_scores: Dict[Tuple[str, str], float] = {}
        
        for seed in seeds:
            try:
                res = graph_engine.rank_related_symbols(seed)
                for item in getattr(res, "items", []):
                    path = item.get("path") or item.get("file_path")
                    name = item.get("name") or item.get("qualname")
                    score = item.get("score", 0.5)
                    
                    if path and name:
                        func_scores[(path, name)] = max(func_scores.get((path, name), 0.0), score)
            except Exception:
                continue
                
        return func_scores

    def rank_files(self, failure_signals: List[FailureSignal], context: Dict[str, Any]) -> Dict[str, float]:
        func_scores = self.rank_functions(failure_signals, context)
        file_scores: Dict[str, float] = {}
        
        for (path, name), score in func_scores.items():
            file_scores[path] = max(file_scores.get(path, 0.0), score)
            
        return file_scores
