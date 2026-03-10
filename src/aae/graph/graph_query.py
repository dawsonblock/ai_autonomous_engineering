from __future__ import annotations

from collections import defaultdict, deque
from typing import Dict, Iterable, List

from aae.contracts.graph import GraphEdgeType, GraphNodeType, GraphQueryResult, GraphSnapshot
from aae.graph.graph_store import SQLiteGraphStore


class GraphQueryEngine:
    def __init__(self, snapshot: GraphSnapshot) -> None:
        self.snapshot = snapshot
        self.nodes = {node.id: node for node in snapshot.nodes}
        self.outgoing: Dict[str, List] = defaultdict(list)
        self.incoming: Dict[str, List] = defaultdict(list)
        for edge in snapshot.edges:
            self.outgoing[edge.source_id].append(edge)
            self.incoming[edge.target_id].append(edge)

    @classmethod
    def from_sqlite(cls, sqlite_path: str) -> "GraphQueryEngine":
        return cls(SQLiteGraphStore(sqlite_path).load())

    def find_functions(self, symbol: str) -> GraphQueryResult:
        symbol_lower = symbol.lower()
        matches = []
        for node in self.nodes.values():
            if node.node_type not in {GraphNodeType.FUNCTION, GraphNodeType.TEST}:
                continue
            haystack = [node.name.lower(), node.qualname.lower()]
            if any(symbol_lower in item for item in haystack):
                matches.append(
                    {
                        "id": node.id,
                        "name": node.name,
                        "qualname": node.qualname,
                        "path": node.path,
                        "line": node.line,
                    }
                )
        return GraphQueryResult(
            query_name="find_functions",
            items=sorted(matches, key=lambda item: (item["path"], item["line"] or 0)),
            summary={"match_count": len(matches), "symbol": symbol},
        )

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
        for function_id in function_ids:
            for edge in self.incoming.get(function_id, []):
                if edge.edge_type != GraphEdgeType.TESTS:
                    continue
                test_node = self.nodes.get(edge.source_id)
                if test_node is None:
                    continue
                tests.append(
                    {
                        "id": test_node.id,
                        "name": test_node.name,
                        "qualname": test_node.qualname,
                        "path": test_node.path,
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
