from __future__ import annotations

from typing import Iterable, List

from aae.contracts.graph import GraphEdge, GraphEdgeType, GraphNode, GraphNodeType
from aae.graph.ast_parser import ParsedPythonFile


class DataflowBuilder:
    def build(self, parsed_files: Iterable[ParsedPythonFile], existing_nodes: list[GraphNode]) -> tuple[list[GraphNode], list[GraphEdge]]:
        nodes = list(existing_nodes)
        node_ids = {node.id for node in nodes}
        edges: List[GraphEdge] = []
        for parsed in parsed_files:
            for source_id, rw in parsed.read_write_sets.items():
                for symbol_name in sorted(set(rw.get("reads", []))):
                    target_id = self._ensure_symbol_node(nodes, node_ids, parsed.file_node.path, symbol_name)
                    edges.append(
                        GraphEdge(
                            source_id=source_id,
                            target_id=target_id,
                            edge_type=GraphEdgeType.READS,
                            metadata={"symbol": symbol_name},
                        )
                    )
                for symbol_name in sorted(set(rw.get("writes", []))):
                    target_id = self._ensure_symbol_node(nodes, node_ids, parsed.file_node.path, symbol_name)
                    edges.append(
                        GraphEdge(
                            source_id=source_id,
                            target_id=target_id,
                            edge_type=GraphEdgeType.WRITES,
                            metadata={"symbol": symbol_name},
                        )
                    )
            for source_id, flows in parsed.param_flows.items():
                for flow in flows:
                    target_id = self._ensure_symbol_node(nodes, node_ids, parsed.file_node.path, flow["target"])
                    edges.append(
                        GraphEdge(
                            source_id=source_id,
                            target_id=target_id,
                            edge_type=GraphEdgeType.PARAM_FLOW,
                            metadata=flow,
                        )
                    )
        return nodes, edges

    def _ensure_symbol_node(self, nodes: list[GraphNode], node_ids: set[str], path: str, symbol_name: str) -> str:
        node_id = "external:symbol:%s:%s" % (path, symbol_name)
        if node_id not in node_ids:
            nodes.append(
                GraphNode(
                    id=node_id,
                    node_type=GraphNodeType.EXTERNAL,
                    name=symbol_name,
                    path=path,
                    qualname=symbol_name,
                    metadata={"external_kind": "symbol"},
                )
            )
            node_ids.add(node_id)
        return node_id
