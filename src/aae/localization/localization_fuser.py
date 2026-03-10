from __future__ import annotations

from typing import Any, Dict, List, Tuple

from .models import CoverageRecord, FailureSignal, LocalizationResult, RankedFile, RankedFunction, StackFrameRef


class LocalizationFuser:
    def fuse(
        self,
        failures: List[FailureSignal],
        stack_frames: List[StackFrameRef],
        coverage: List[CoverageRecord],
        line_scores: Dict[Tuple[str, int], float],
        func_spectrum_scores: Dict[Tuple[str, str], float],
        file_spectrum_scores: Dict[str, float],
        graph_func_scores: Dict[Tuple[str, str], float],
        graph_file_scores: Dict[str, float],
        context: Dict[str, Any],
    ) -> LocalizationResult:
        settings = context.get("localization_settings", {})
        spectrum_weight = settings.get("spectrum_weight", 0.7)
        graph_weight = settings.get("graph_proximity_weight", 0.3)
        limit_files = settings.get("top_files", 3)
        limit_funcs = settings.get("top_functions", 5)

        files_map: Dict[str, float] = {}
        for file_path, score in file_spectrum_scores.items():
            files_map[file_path] = files_map.get(file_path, 0.0) + score * spectrum_weight
        for file_path, score in graph_file_scores.items():
            files_map[file_path] = files_map.get(file_path, 0.0) + score * graph_weight
            
        ranked_files = [
            RankedFile(file_path=path, score=score, reasons=[f"Fused score: {score:.3f}"])
            for path, score in sorted(files_map.items(), key=lambda x: x[1], reverse=True)[:limit_files]
        ]
        
        func_map: Dict[Tuple[str, str], float] = {}
        for key, score in func_spectrum_scores.items():
            func_map[key] = func_map.get(key, 0.0) + score * spectrum_weight
        for key, score in graph_func_scores.items():
            func_map[key] = func_map.get(key, 0.0) + score * graph_weight
            
        ranked_funcs = [
            RankedFunction(file_path=path, function_name=name, score=score, reasons=[f"Fused score: {score:.3f}"])
            for (path, name), score in sorted(func_map.items(), key=lambda x: x[1], reverse=True)[:limit_funcs]
        ]
        
        return LocalizationResult(
            files=ranked_files,
            functions=ranked_funcs,
            spans=[]
        )
