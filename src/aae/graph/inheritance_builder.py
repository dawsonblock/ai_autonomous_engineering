from __future__ import annotations

from typing import Dict, Iterable, List

from aae.contracts.graph import GraphEdge, GraphEdgeType, GraphNode, GraphNodeType, SymbolDefinition
from aae.graph.ast_parser import ParsedPythonFile


class InheritanceBuilder:
    def build(
        self,
        parsed_files: Iterable[ParsedPythonFile],
        definitions: list[SymbolDefinition],
        existing_nodes: list[GraphNode],
    ) -> tuple[list[GraphNode], list[GraphEdge]]:
        nodes = list(existing_nodes)
        edges: List[GraphEdge] = []
        class_by_name = {
            definition.name: definition
            for definition in definitions
            if definition.symbol_type == "class"
        }
        methods_by_class: Dict[str, list[str]] = {}
        method_symbols: Dict[tuple[str, str], SymbolDefinition] = {}
        for definition in definitions:
            if definition.class_scope:
                methods_by_class.setdefault(definition.class_scope, []).append(definition.name)
                method_symbols[(definition.class_scope, definition.name)] = definition

        existing_ids = {node.id for node in nodes}
        for parsed in parsed_files:
            for class_id, base_names in parsed.class_bases.items():
                for base_name in base_names:
                    base_def = class_by_name.get(base_name.split(".")[-1])
                    if base_def is None:
                        external_id = "external:class:%s" % base_name
                        if external_id not in existing_ids:
                            nodes.append(
                                GraphNode(
                                    id=external_id,
                                    node_type=GraphNodeType.EXTERNAL,
                                    name=base_name.split(".")[-1],
                                    qualname=base_name,
                                    metadata={"external_kind": "class"},
                                )
                            )
                            existing_ids.add(external_id)
                        target_id = external_id
                    else:
                        target_id = base_def.symbol_id
                    edges.append(
                        GraphEdge(
                            source_id=class_id,
                            target_id=target_id,
                            edge_type=GraphEdgeType.INHERITS,
                            metadata={"base_name": base_name},
                        )
                    )
        for definition in definitions:
            if not definition.class_scope:
                continue
            class_node_id = "class:%s" % definition.class_scope
            for edge in [edge for edge in edges if edge.source_id == class_node_id and edge.edge_type == GraphEdgeType.INHERITS]:
                base_scope = edge.metadata.get("base_name", "").split(".")[-1]
                base_class = class_by_name.get(base_scope)
                if base_class is None:
                    continue
                parent_method = method_symbols.get((base_class.qualname, definition.name))
                if parent_method is None:
                    continue
                edges.append(
                    GraphEdge(
                        source_id=definition.symbol_id,
                        target_id=parent_method.symbol_id,
                        edge_type=GraphEdgeType.OVERRIDES,
                        metadata={"class_scope": definition.class_scope},
                    )
                )
        return nodes, edges
