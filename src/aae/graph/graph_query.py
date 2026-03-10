from __future__ import annotations

from collections import defaultdict, deque
from typing import Dict, Iterable, List

from aae.code_analysis.symbol_index import SymbolIndex
from aae.contracts.graph import GraphEdgeType, GraphNodeType, GraphQueryResult, GraphSnapshot
from aae.graph.graph_store import SQLiteGraphStore


class GraphQueryEngine:
    def __init__(self, snapshot: GraphSnapshot) -> None:
        self.snapshot = snapshot
        self.nodes = {node.id: node for node in snapshot.nodes}
        self.symbols = {symbol.symbol_id: symbol for symbol in snapshot.symbols}
        self.outgoing: Dict[str, List] = defaultdict(list)
        self.incoming: Dict[str, List] = defaultdict(list)
        for edge in snapshot.edges:
            self.outgoing[edge.source_id].append(edge)
            self.incoming[edge.target_id].append(edge)
        self.symbol_index = SymbolIndex.from_snapshot(snapshot)

    @classmethod
    def from_sqlite(cls, sqlite_path: str) -> "GraphQueryEngine":
        return cls(SQLiteGraphStore(sqlite_path).load())

    def find_functions(self, symbol: str) -> GraphQueryResult:
        definitions = self.symbol_index.lookup(symbol)
        seen = set()
        matches = []
        for definition in definitions:
            if definition.symbol_type not in {"function", "test"}:
                continue
            if definition.symbol_id in seen:
                continue
            seen.add(definition.symbol_id)
            matches.append(
                {
                    "id": definition.symbol_id,
                    "name": definition.name,
                    "qualname": definition.qualname,
                    "path": definition.file_path,
                    "line": definition.line,
                    "signature": definition.signature,
                    "class_scope": definition.class_scope,
                }
            )
            for edge in self.outgoing.get(definition.symbol_id, []):
                if edge.edge_type != GraphEdgeType.OVERRIDES:
                    continue
                overridden = self.symbols.get(edge.target_id)
                if overridden is None or overridden.symbol_id in seen:
                    continue
                seen.add(overridden.symbol_id)
                matches.append(
                    {
                        "id": overridden.symbol_id,
                        "name": overridden.name,
                        "qualname": overridden.qualname,
                        "path": overridden.file_path,
                        "line": overridden.line,
                        "signature": overridden.signature,
                        "class_scope": overridden.class_scope,
                    }
                )
        return GraphQueryResult(query_name="find_functions", items=sorted(matches, key=lambda item: (item["path"], item["line"] or 0)), summary={"match_count": len(matches), "symbol": symbol})

    def trace_call_chain(self, symbol: str, max_depth: int = 5) -> GraphQueryResult:
        start_items = self.find_functions(symbol).items
        paths: List[List[str]] = []
        for item in start_items[:3]:
            start_id = item["id"]
            queue = deque([(start_id, [self.nodes[start_id].qualname], 0)])
            while queue:
                node_id, path, depth = queue.popleft()
                if depth >= max_depth:
                    paths.append(path)
                    continue
                call_edges = [edge for edge in self.outgoing[node_id] if edge.edge_type == GraphEdgeType.CALLS]
                call_edges.extend([edge for edge in self.outgoing[node_id] if edge.edge_type == GraphEdgeType.OVERRIDES])
                if not call_edges:
                    paths.append(path)
                    continue
                for edge in call_edges:
                    target = self.nodes.get(edge.target_id)
                    if target is None or target.qualname in path:
                        continue
                    queue.append((edge.target_id, path + [target.qualname], depth + 1))
        return GraphQueryResult(
            query_name="trace_call_chain",
            paths=paths,
            summary={"path_count": len(paths), "symbol": symbol},
        )

    def tests_covering_function(self, symbol: str) -> GraphQueryResult:
        functions = self.find_functions(symbol).items
        function_ids = {item["id"] for item in functions}
        tests = []
        seen = set()
        for function_id in function_ids:
            for association in self.snapshot.coverage:
                if association.target_symbol_id != function_id:
                    continue
                test_node = self.nodes.get(association.test_node_id)
                if test_node is None:
                    continue
                key = (test_node.id, association.source)
                if key in seen:
                    continue
                seen.add(key)
                tests.append(
                    {
                        "id": test_node.id,
                        "name": test_node.name,
                        "qualname": test_node.qualname,
                        "path": test_node.path,
                        "source": association.source,
                        "confidence": association.confidence,
                    }
                )
            for edge in self.incoming.get(function_id, []):
                if edge.edge_type != GraphEdgeType.TESTS:
                    continue
                test_node = self.nodes.get(edge.source_id)
                if test_node is None:
                    continue
                key = (test_node.id, edge.metadata.get("source", "graph"))
                if key in seen:
                    continue
                seen.add(key)
                tests.append(
                    {
                        "id": test_node.id,
                        "name": test_node.name,
                        "qualname": test_node.qualname,
                        "path": test_node.path,
                        "source": edge.metadata.get("source", "graph"),
                        "confidence": edge.metadata.get("confidence", 0.5),
                    }
                )
        return GraphQueryResult(
            query_name="tests_covering_function",
            items=sorted(tests, key=lambda item: (item["path"], item["qualname"])),
            summary={"test_count": len(tests), "symbol": symbol},
        )

    def files_importing(self, module: str) -> GraphQueryResult:
        matches = []
        for node in self.nodes.values():
            if node.node_type != GraphNodeType.FILE:
                continue
            for edge in self.outgoing.get(node.id, []):
                if edge.edge_type != GraphEdgeType.IMPORTS:
                    continue
                imported = edge.metadata.get("module", "")
                if imported == module or imported.endswith(".%s" % module) or module in imported:
                    matches.append({"path": node.path, "module": imported})
        return GraphQueryResult(
            query_name="files_importing",
            items=sorted(matches, key=lambda item: item["path"]),
            summary={"file_count": len(matches), "module": module},
        )

    def find_references(self, symbol: str) -> GraphQueryResult:
        references = []
        for reference in self.symbol_index.find_references(symbol):
            references.append(
                {
                    "source_symbol_id": reference.source_symbol_id,
                    "resolved_symbol_id": reference.resolved_symbol_id,
                    "referenced_name": reference.referenced_name,
                    "file_path": reference.file_path,
                    "line": reference.line,
                    "reference_type": reference.reference_type,
                }
            )
        return GraphQueryResult(
            query_name="find_references",
            items=references,
            summary={"reference_count": len(references), "symbol": symbol},
        )

    def rank_related_symbols(self, symbol: str) -> GraphQueryResult:
        items = list(self.symbol_index.rank_related_symbols(symbol))
        return GraphQueryResult(
            query_name="rank_related_symbols",
            items=items,
            summary={"related_count": len(items), "symbol": symbol},
        )
