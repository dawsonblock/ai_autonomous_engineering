from __future__ import annotations

import math
from collections import defaultdict
from typing import Any, Dict, List, Tuple

from .models import CoverageRecord, FailureSignal


class SpectrumRanker:
    def score(
        self, failed_records: List[CoverageRecord], passed_records: List[CoverageRecord]
    ) -> Dict[Tuple[str, int], float]:
        failed_hits: Dict[Tuple[str, int], int] = defaultdict(int)
        passed_hits: Dict[Tuple[str, int], int] = defaultdict(int)

        for rec in failed_records:
            for line in rec.line_hits:
                failed_hits[(rec.file_path, line)] += 1

        for rec in passed_records:
            for line in rec.line_hits:
                passed_hits[(rec.file_path, line)] += 1

        total_failed = max(len(failed_records), 1)
        scores: Dict[Tuple[str, int], float] = {}

        for key, fh in failed_hits.items():
            ph = passed_hits.get(key, 0)
            scores[key] = fh / math.sqrt(total_failed * (fh + ph))

        return scores

    def rank(self, failure_signals: List[FailureSignal], context: Dict[str, Any]) -> Dict[Tuple[str, int], float]:
        return self.rank_lines(failure_signals, context)

    def rank_lines(self, failure_signals: List[FailureSignal], context: Dict[str, Any]) -> Dict[Tuple[str, int], float]:
        coverage_records: List[CoverageRecord] = context.get("coverage_records", [])

        failed_test_names = {f.test_name for f in failure_signals}
        failed_records = [r for r in coverage_records if r.test_name in failed_test_names]
        passed_records = [r for r in coverage_records if r.test_name not in failed_test_names]

        return self.score(failed_records, passed_records)

    def rank_functions(self, failure_signals: List[FailureSignal], context: Dict[str, Any]) -> Dict[Tuple[str, str], float]:
        coverage_records: List[CoverageRecord] = context.get("coverage_records", [])

        failed_test_names = {f.test_name for f in failure_signals}
        failed_records = [r for r in coverage_records if r.test_name in failed_test_names]
        passed_records = [r for r in coverage_records if r.test_name not in failed_test_names]
        
        line_scores = self.score(failed_records, passed_records)
        
        fn_scores_temp: Dict[Tuple[str, str], List[float]] = defaultdict(list)
        for rec in failed_records + passed_records:
            if rec.function_name:
                for line in rec.line_hits:
                    key = (rec.file_path, line)
                    if key in line_scores:
                        fn_scores_temp[(rec.file_path, rec.function_name)].append(line_scores[key])
                        
        fn_scores: Dict[Tuple[str, str], float] = {}
        for fn_key, scores in fn_scores_temp.items():
            if scores:
                fn_scores[fn_key] = max(scores)
                
        return fn_scores

    def rank_files(self, failure_signals: List[FailureSignal], context: Dict[str, Any]) -> Dict[str, float]:
        line_scores = self.rank_lines(failure_signals, context)
        
        file_scores_temp: Dict[str, List[float]] = defaultdict(list)
        for (file_path, _), score in line_scores.items():
            file_scores_temp[file_path].append(score)
            
        file_scores: Dict[str, float] = {}
        for file_path, scores in file_scores_temp.items():
            if scores:
                file_scores[file_path] = max(scores)
                
        return file_scores
