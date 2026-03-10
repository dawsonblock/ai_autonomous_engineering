from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent
from aae.bug_localization.suspiciousness_ranker import SuspiciousnessRanker
from aae.bug_localization.test_failure_mapper import TestFailureMapper


class BugLocalizerAgent(BaseMicroAgent):
    name = "bug_localizer"

    def __init__(
        self,
        failure_mapper: TestFailureMapper | None = None,
        ranker: SuspiciousnessRanker | None = None,
    ) -> None:
        self.failure_mapper = failure_mapper or TestFailureMapper()
        self.ranker = ranker or SuspiciousnessRanker()

    async def run(self, task, context):
        graph = context["graph"]
        behavior_engine = context.get("behavior_engine")
        graph_context = context.get("graph_context", {})
        candidate_symbols = [entry.get("symbol", "") for entry in graph_context.get("symbol_context", [])]
        if not candidate_symbols:
            candidate_symbols = [symbol.get("name", "") for symbol in context.get("symbols", [])]
        evidence = self.failure_mapper.map_failures(
            {
                "covering_tests": context.get("covering_tests") or graph_context.get("covering_tests", []),
                "impacted_tests": context.get("impacted_tests", []),
                "recent_failures": context.get("recent_failures", []),
                "sandbox_runs": context.get("sandbox_runs", []),
                "trace_records": context.get("trace_records", []),
            }
        )
        localization = self.ranker.rank(graph, behavior_engine, candidate_symbols, evidence)
        candidate_files = [
            {
                "path": location.file_path,
                "reason": ",".join(location.evidence_sources) or "suspiciousness_rank",
                "score": location.confidence,
            }
            for location in localization.suspicious_locations
        ]
        if not candidate_files:
            candidate_files = list(context.get("candidate_files", []))
        root_cause_symbol = localization.suspicious_locations[0].symbol if localization.suspicious_locations else ""
        if not root_cause_symbol and context.get("symbols"):
            root_cause_symbol = context["symbols"][0].get("name", "")
        return {
            "candidate_files": candidate_files[:4],
            "localized_files": candidate_files[:4],
            "suspicious_locations": [location.model_dump(mode="json") for location in localization.suspicious_locations[:8]],
            "evidence": [item.model_dump(mode="json") for item in localization.evidence[:12]],
            "root_cause_symbol": root_cause_symbol,
            "confidence": localization.suspicious_locations[0].confidence if localization.suspicious_locations else (0.72 if candidate_files else 0.35),
        }
