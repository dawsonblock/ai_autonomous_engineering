from __future__ import annotations

from collections import defaultdict
from typing import Dict, Iterable, List

from aae.contracts.graph import GraphEdge, GraphEdgeType, GraphNodeType
from aae.graph.ast_parser import ParsedPythonFile


class CallGraphBuilder:
    def build_edges(self, parsed_files: Iterable[ParsedPythonFile]) -> List[GraphEdge]:
        symbol_index: Dict[str, List[str]] = defaultdict(list)
        node_type_by_id: Dict[str, GraphNodeType] = {}
        for parsed in parsed_files:
            for node in parsed.nodes:
                node_type_by_id[node.id] = node.node_type
                if node.node_type in {GraphNodeType.FUNCTION, GraphNodeType.TEST}:
                    symbol_index[node.name].append(node.id)
                    symbol_index[node.qualname.split(".")[-1]].append(node.id)
                    symbol_index[node.qualname].append(node.id)

        edges: List[GraphEdge] = []
        seen = set()
        for parsed in parsed_files:
            for source_id, call_names in parsed.calls_by_node.items():
                source_type = node_type_by_id.get(source_id, GraphNodeType.FUNCTION)
                for call_name in call_names:
                    simple = call_name.split(".")[-1]
                    candidates = symbol_index.get(call_name, []) or symbol_index.get(simple, [])
                    for target_id in candidates:
                        edge_type = GraphEdgeType.TESTS if source_type == GraphNodeType.TEST else GraphEdgeType.CALLS
                        key = (source_id, target_id, edge_type.value)
                        if key in seen or source_id == target_id:
                            continue
                        seen.add(key)
                        edges.append(
                            GraphEdge(
                                source_id=source_id,
                                target_id=target_id,
                                edge_type=edge_type,
                                metadata={"resolved_from": call_name},
                            )
                        )
        return edges
