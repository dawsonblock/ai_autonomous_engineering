from __future__ import annotations

from typing import Dict, List, Tuple

from .models import RankedFunction, RankedSpan


class EditSpanLocator:
    def locate(
        self,
        ranked_functions: List[RankedFunction],
        line_scores: Dict[Tuple[str, int], float],
        ast_spans: List[RankedSpan],
        failure_type: str,
    ) -> List[RankedSpan]:
        spans: List[RankedSpan] = []
        
        for span in ast_spans:
            span_lines = range(span.start_line, span.end_line + 1)
            scores = [line_scores.get((span.file_path, line_num), 0.0) for line_num in span_lines]
            if not any(scores):
                continue
            
            avg_score = sum(scores) / len(scores)
            max_score = max(scores)
            
            final_score = avg_score * 0.5 + max_score * 0.5
            if final_score > 0:
                span.score = final_score
                span.evidence["failure_type"] = failure_type
                span.reasons.append(f"Contains {len([s for s in scores if s > 0])} suspicious lines")
                spans.append(span)
                
        spans.sort(key=lambda s: s.score, reverse=True)
        return spans
