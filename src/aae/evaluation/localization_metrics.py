from __future__ import annotations

from typing import Any, Dict, Optional, Tuple

from aae.localization.models import LocalizationResult


class LocalizationMetrics:
    @staticmethod
    def calculate(
        result: LocalizationResult,
        true_file: str,
        true_function: Optional[str] = None,
        true_span: Optional[Tuple[int, int]] = None,
    ) -> Dict[str, Any]:
        metrics = {
            "correct_file_in_top_1": False,
            "correct_file_in_top_3": False,
            "correct_file_in_top_5": False,
            "correct_function_in_top_1": False,
            "correct_function_in_top_3": False,
            "edit_span_overlap_iou": 0.0,
            "span_contains_true_edited_lines": False,
            "total_files_passed_to_patcher": len(result.files),
            "total_localized_spans": len(result.spans),
        }

        # Check files
        top_files = [f.file_path for f in result.files]
        if true_file in top_files:
            idx = top_files.index(true_file)
            if idx < 1:
                metrics["correct_file_in_top_1"] = True
            if idx < 3:
                metrics["correct_file_in_top_3"] = True
            if idx < 5:
                metrics["correct_file_in_top_5"] = True

        # Check functions
        if true_function:
            top_funcs = [f.function_name for f in result.functions]
            if true_function in top_funcs:
                idx = top_funcs.index(true_function)
                if idx < 1:
                    metrics["correct_function_in_top_1"] = True
                if idx < 3:
                    metrics["correct_function_in_top_3"] = True

        # Check spans
        if true_span:
            for span in result.spans:
                if span.file_path == true_file:
                    overlap_start = max(span.start_line, true_span[0])
                    overlap_end = min(span.end_line, true_span[1])
                    if overlap_start <= overlap_end:
                        metrics["span_contains_true_edited_lines"] = True
                        intersection = overlap_end - overlap_start + 1
                        union = (span.end_line - span.start_line + 1) + (true_span[1] - true_span[0] + 1) - intersection
                        iou = intersection / union
                        if iou > metrics["edit_span_overlap_iou"]:
                            metrics["edit_span_overlap_iou"] = iou

        return metrics
