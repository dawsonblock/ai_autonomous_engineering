from __future__ import annotations

from aae.behavior_model.behavior_query_engine import BehaviorQueryEngine
from aae.contracts.planner import DependencyImpactResult
from aae.graph.graph_query import GraphQueryEngine


class DependencyImpactAnalyzer:
    def analyze(
        self,
        graph: GraphQueryEngine,
        changed_files: list[str],
        behavior: BehaviorQueryEngine | None = None,
    ) -> DependencyImpactResult:
        affected_functions = []
        affected_symbols = []
        impacted_files = set(changed_files)
        for node in graph.nodes.values():
            if node.path in changed_files and node.node_type.value in {"function", "test"}:
                affected_functions.append(node.qualname)
                affected_symbols.append(node.name)
                impacted_files.add(node.path)
        if behavior is not None:
            for file_match in behavior.suspicious_files(affected_symbols or changed_files).items[:8]:
                impacted_files.add(file_match["path"])
            for function_name in list(affected_functions):
                for path_result in behavior.causal_path(function_name.split(".")[-1]).items[:3]:
                    for step in path_result["path"]:
                        affected_functions.append(step)
                        affected_symbols.append(step.split(".")[-1])
        return DependencyImpactResult(
            affected_functions=sorted(set(affected_functions)),
            affected_symbols=sorted(set(affected_symbols)),
            impacted_files=sorted(impacted_files),
            impact_size=len(set(affected_functions)) + len(impacted_files),
        )
