from __future__ import annotations

import re
from typing import Dict, List

from aae.code_analysis.context_ranker import ContextRanker
from aae.graph.graph_query import GraphQueryEngine


def find_functions(graph: GraphQueryEngine, symbol: str) -> List[Dict[str, object]]:
    return graph.find_functions(symbol).items


def trace_call_chain(graph: GraphQueryEngine, symbol: str, max_depth: int = 5) -> List[str]:
    return [" -> ".join(path) for path in graph.trace_call_chain(symbol, max_depth=max_depth).paths]


def tests_covering_function(graph: GraphQueryEngine, symbol: str) -> List[Dict[str, object]]:
    return graph.tests_covering_function(symbol).items


def files_importing(graph: GraphQueryEngine, module: str) -> List[Dict[str, object]]:
    return graph.files_importing(module).items


class GraphContextBuilder:
    def __init__(self, graph: GraphQueryEngine, context_ranker: ContextRanker | None = None) -> None:
        self.graph = graph
        self.context_ranker = context_ranker or ContextRanker()

    def build(self, goal: str, behavior_context: Dict[str, object] | None = None, failure_evidence: List[Dict[str, object]] | None = None) -> Dict[str, object]:
        symbols = _candidate_symbols(goal)
        symbol_context = []
        call_chains = []
        covering_tests = []
        imported_files = []
        reference_context = []

        for symbol in symbols[:3]:
            matches = find_functions(self.graph, symbol)
            if matches:
                symbol_context.append({"symbol": symbol, "matches": matches[:3]})
            chains = trace_call_chain(self.graph, symbol, max_depth=4)
            if chains:
                call_chains.extend(chains[:3])
            tests = tests_covering_function(self.graph, symbol)
            if tests:
                covering_tests.extend(test["path"] for test in tests[:3])
            imports = files_importing(self.graph, symbol)
            if imports:
                imported_files.extend(item["path"] for item in imports[:3])
            references = self.graph.find_references(symbol).items
            if references:
                reference_context.append({"symbol": symbol, "references": references[:8]})

        graph_context = {
            "candidate_symbols": symbols,
            "symbol_context": symbol_context,
            "call_chains": sorted(set(call_chains)),
            "covering_tests": sorted(set(covering_tests)),
            "imported_files": sorted(set(imported_files)),
            "reference_context": reference_context,
        }
        ranked = self.context_ranker.rank(goal, self.graph, graph_context, behavior_context=behavior_context or {}, failure_evidence=failure_evidence or [])
        graph_context.update(ranked)
        return graph_context


def _candidate_symbols(goal: str) -> List[str]:
    tokens = re.findall(r"[A-Za-z_][A-Za-z0-9_]{2,}", goal or "")
    seen = []
    for token in tokens:
        lowered = token.lower()
        if lowered in {"the", "and", "with", "from", "into", "that", "this", "fix", "build"}:
            continue
        if token not in seen:
            seen.append(token)
    return seen
