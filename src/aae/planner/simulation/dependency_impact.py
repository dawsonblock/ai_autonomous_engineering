from __future__ import annotations

from aae.contracts.planner import DependencyImpactResult
from aae.graph.graph_query import GraphQueryEngine


class DependencyImpactAnalyzer:
    def analyze(self, graph: GraphQueryEngine, changed_files: list[str]) -> DependencyImpactResult:
        affected_functions = []
        affected_symbols = []
        impacted_files = set(changed_files)
        for node in graph.nodes.values():
            if node.path in changed_files and node.node_type.value in {"function", "test"}:
                affected_functions.append(node.qualname)
                affected_symbols.append(node.name)
                impacted_files.add(node.path)
        return DependencyImpactResult(
            affected_functions=sorted(set(affected_functions)),
            affected_symbols=sorted(set(affected_symbols)),
            impacted_files=sorted(impacted_files),
            impact_size=len(set(affected_functions)) + len(impacted_files),
        )
