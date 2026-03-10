from __future__ import annotations

from typing import Any, Dict

from .ast_span_extractor import ASTSpanExtractor
from .coverage_loader import CoverageLoader
from .edit_span_locator import EditSpanLocator
from .failure_mapper import FailureMapper
from .graph_proximity_ranker import GraphProximityRanker
from .localization_fuser import LocalizationFuser
from .models import LocalizationResult
from .semantic_ranker import SemanticRanker
from .spectrum_ranker import SpectrumRanker
from .stacktrace_linker import StacktraceLinker
from .trace_ranker import TraceRanker


class LocalizationService:
    def __init__(
        self,
        coverage_loader: CoverageLoader | None = None,
        graph_ranker: GraphProximityRanker | None = None,
        trace_ranker: TraceRanker | None = None,
        semantic_ranker: SemanticRanker | None = None,
        spectrum_ranker: SpectrumRanker | None = None,
    ):
        self.failure_mapper = FailureMapper()
        self.stacktrace_linker = StacktraceLinker()
        self.coverage_loader = coverage_loader or CoverageLoader()
        self.graph_ranker = graph_ranker or GraphProximityRanker()
        self.trace_ranker = trace_ranker or TraceRanker()
        self.semantic_ranker = semantic_ranker or SemanticRanker()
        self.spectrum_ranker = spectrum_ranker or SpectrumRanker()
        self.ast_span_extractor = ASTSpanExtractor()
        self.edit_span_locator = EditSpanLocator()
        self.fuser = LocalizationFuser()

    def localize(self, raw_test_output: str, repo_root: str, context: Dict[str, Any]) -> LocalizationResult:
        failures = self.failure_mapper.parse_pytest_output(raw_test_output)

        stack_frames = []
        for failure in failures:
            if failure.stacktrace:
                stack_frames.extend(self.stacktrace_linker.parse(failure.stacktrace, repo_root))

        coverage = self.coverage_loader.load(context)
        context["coverage_records"] = coverage
        context["stacktrace_frames"] = stack_frames
        
        # Spectrum Ranking
        line_scores = self.spectrum_ranker.rank_lines(failures, context)
        func_spectrum_scores = self.spectrum_ranker.rank_functions(failures, context)
        file_spectrum_scores = self.spectrum_ranker.rank_files(failures, context)
        
        context["spectrum_functions"] = func_spectrum_scores

        # Graph Ranking
        graph_func_scores = self.graph_ranker.rank_functions(failures, context)
        graph_file_scores = self.graph_ranker.rank_files(failures, context)

        # Fuse
        fused_result = self.fuser.fuse(
            failures=failures,
            stack_frames=stack_frames,
            coverage=coverage,
            line_scores=line_scores,
            func_spectrum_scores=func_spectrum_scores,
            file_spectrum_scores=file_spectrum_scores,
            graph_func_scores=graph_func_scores,
            graph_file_scores=graph_file_scores,
            context=context,
        )
        
        # Extract spans
        ast_spans = []
        for func in fused_result.functions:
            ast_spans.extend(self.ast_span_extractor.extract_spans(func.file_path, repo_root, func.function_name))
            
        settings = context.get("localization_settings", {})
        limit_spans = settings.get("top_spans", 10)
        
        failure_type = "unknown"
        if failures and failures[0].exception_type is not None:
            failure_type = failures[0].exception_type

        fused_result.spans = self.edit_span_locator.locate(
            ranked_functions=fused_result.functions,
            line_scores=line_scores,
            ast_spans=ast_spans,
            failure_type=failure_type
        )[:limit_spans]

        return fused_result
