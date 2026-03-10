from __future__ import annotations

import re
from typing import Dict, List

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
    def __init__(self, graph: GraphQueryEngine) -> None:
        self.graph = graph

    def build(self, goal: str) -> Dict[str, object]:
        symbols = _candidate_symbols(goal)
        symbol_context = []
        call_chains = []
        covering_tests = []
        imported_files = []

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

        return {
            "candidate_symbols": symbols,
            "symbol_context": symbol_context,
            "call_chains": sorted(set(call_chains)),
            "covering_tests": sorted(set(covering_tests)),
            "imported_files": sorted(set(imported_files)),
        }


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
