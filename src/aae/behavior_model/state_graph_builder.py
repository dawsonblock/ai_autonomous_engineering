from __future__ import annotations

from collections import defaultdict
from typing import Dict, Iterable, List

from aae.behavior_model.cfg_builder import BehaviorCfgBuilder
from aae.contracts.behavior import (
    BehaviorEdge,
    BehaviorEdgeType,
    BehaviorNode,
    BehaviorNodeType,
    BehaviorSnapshot,
    StateTransition,
)
from aae.contracts.graph import GraphEdgeType, GraphSnapshot


class StateGraphBuilder:
    def __init__(self, cfg_builder: BehaviorCfgBuilder | None = None) -> None:
        self.cfg_builder = cfg_builder or BehaviorCfgBuilder()

    def build(self, repo_path: str, graph_snapshot: GraphSnapshot) -> BehaviorSnapshot:
        nodes: List[BehaviorNode] = []
        edges: List[BehaviorEdge] = []
        transitions: List[StateTransition] = []
        seen_nodes = set()
        symbol_map = {symbol.symbol_id: symbol for symbol in graph_snapshot.symbols}
        reads_by_symbol = defaultdict(list)
        writes_by_symbol = defaultdict(list)
        params_by_symbol = defaultdict(list)

        for edge in graph_snapshot.edges:
            if edge.edge_type == GraphEdgeType.READS:
                reads_by_symbol[edge.source_id].append(edge.target_id)
            elif edge.edge_type == GraphEdgeType.WRITES:
                writes_by_symbol[edge.source_id].append(edge.target_id)
            elif edge.edge_type == GraphEdgeType.PARAM_FLOW:
                params_by_symbol[edge.source_id].append(edge.target_id)

        for symbol in graph_snapshot.symbols:
            if symbol.symbol_type not in {"function", "test"}:
                continue
            cfg_summary = self.cfg_builder.build(repo_path, symbol.file_path, symbol.name)
            function_node = BehaviorNode(
                id="behavior:%s" % symbol.symbol_id,
                node_type=BehaviorNodeType.FUNCTION,
                name=symbol.name,
                path=symbol.file_path,
                qualname=symbol.qualname,
                line=symbol.line,
                metadata={
                    "signature": symbol.signature,
                    "cfg": cfg_summary,
                },
            )
            self._add_node(nodes, seen_nodes, function_node)

            for parameter in symbol.metadata.get("parameters", []):
                input_node = BehaviorNode(
                    id="input:%s:%s" % (symbol.symbol_id, parameter),
                    node_type=BehaviorNodeType.INPUT,
                    name=parameter,
                    path=symbol.file_path,
                    qualname="%s.%s" % (symbol.qualname, parameter),
                )
                self._add_node(nodes, seen_nodes, input_node)
                edges.append(
                    BehaviorEdge(
                        source_id=input_node.id,
                        target_id=function_node.id,
                        edge_type=BehaviorEdgeType.CONSUMES,
                        metadata={"kind": "parameter"},
                    )
                )

            output_node = BehaviorNode(
                id="output:%s" % symbol.symbol_id,
                node_type=BehaviorNodeType.OUTPUT,
                name="%s.output" % symbol.name,
                path=symbol.file_path,
                qualname="%s.output" % symbol.qualname,
                metadata={"return_lines": cfg_summary.get("returns", [])},
            )
            self._add_node(nodes, seen_nodes, output_node)
            edges.append(
                BehaviorEdge(
                    source_id=function_node.id,
                    target_id=output_node.id,
                    edge_type=BehaviorEdgeType.RETURNS,
                )
            )
            transitions.append(
                StateTransition(
                    transition_id="transition:%s:return" % symbol.symbol_id,
                    source_state=function_node.id,
                    target_state=output_node.id,
                    trigger="return",
                )
            )

        for edge in graph_snapshot.edges:
            if edge.edge_type == GraphEdgeType.CALLS:
                edges.append(
                    BehaviorEdge(
                        source_id="behavior:%s" % edge.source_id,
                        target_id="behavior:%s" % edge.target_id,
                        edge_type=BehaviorEdgeType.CALLS,
                        metadata=edge.metadata,
                    )
                )

        for symbol_id, references in reads_by_symbol.items():
            for target_id in references:
                state_node = self._variable_node(symbol_id, target_id, symbol_map, "read")
                self._add_node(nodes, seen_nodes, state_node)
                edges.append(
                    BehaviorEdge(
                        source_id="behavior:%s" % symbol_id,
                        target_id=state_node.id,
                        edge_type=BehaviorEdgeType.CONSUMES,
                        metadata={"kind": "read"},
                    )
                )

        for symbol_id, references in writes_by_symbol.items():
            for target_id in references:
                state_node = self._variable_node(symbol_id, target_id, symbol_map, "write")
                self._add_node(nodes, seen_nodes, state_node)
                edges.append(
                    BehaviorEdge(
                        source_id="behavior:%s" % symbol_id,
                        target_id=state_node.id,
                        edge_type=BehaviorEdgeType.MODIFIES,
                        metadata={"kind": "write"},
                    )
                )

        for symbol_id, targets in params_by_symbol.items():
            for target_id in targets:
                target_symbol = symbol_map.get(target_id)
                if target_symbol is None:
                    continue
                edges.append(
                    BehaviorEdge(
                        source_id="behavior:%s" % symbol_id,
                        target_id="behavior:%s" % target_id,
                        edge_type=BehaviorEdgeType.PRODUCES,
                        metadata={"kind": "param_flow", "target_symbol": target_symbol.qualname},
                    )
                )

        return BehaviorSnapshot(
            root_path=repo_path,
            nodes=nodes,
            edges=edges,
            transitions=transitions,
            metadata={
                "function_count": len([node for node in nodes if node.node_type == BehaviorNodeType.FUNCTION]),
                "state_count": len([node for node in nodes if node.node_type == BehaviorNodeType.STATE]),
            },
        )

    def _variable_node(
        self,
        symbol_id: str,
        target_id: str,
        symbol_map: Dict[str, object],
        suffix: str,
    ) -> BehaviorNode:
        target_symbol = symbol_map.get(target_id)
        if target_symbol is not None:
            name = target_symbol.name
            qualname = target_symbol.qualname
            path = target_symbol.file_path
        else:
            name = target_id.split(":")[-1]
            qualname = target_id
            path = ""
        return BehaviorNode(
            id="state:%s:%s:%s" % (symbol_id, suffix, name),
            node_type=BehaviorNodeType.STATE,
            name=name,
            path=path,
            qualname=qualname,
            metadata={"source_symbol_id": symbol_id},
        )

    def _add_node(self, nodes: List[BehaviorNode], seen_nodes: set[str], node: BehaviorNode) -> None:
        if node.id in seen_nodes:
            return
        seen_nodes.add(node.id)
        nodes.append(node)
