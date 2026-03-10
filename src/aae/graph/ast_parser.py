from __future__ import annotations

import ast
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List

from aae.contracts.graph import GraphEdge, GraphEdgeType, GraphNode, GraphNodeType
from aae.graph.dependency_extractor import is_python_test_path, module_name_from_path, normalize_import_name


@dataclass
class ParsedPythonFile:
    file_node: GraphNode
    module_node: GraphNode
    nodes: List[GraphNode] = field(default_factory=list)
    edges: List[GraphEdge] = field(default_factory=list)
    imports: List[str] = field(default_factory=list)
    calls_by_node: Dict[str, List[str]] = field(default_factory=dict)


class PythonAstParser:
    def parse_file(self, root_path: str, file_path: str) -> ParsedPythonFile:
        root = Path(root_path).resolve()
        file_obj = Path(file_path).resolve()
        relative_path = file_obj.relative_to(root)
        relative = relative_path.as_posix()
        module_name = module_name_from_path(root, file_obj)
        is_test_file = is_python_test_path(relative_path)

        file_node = GraphNode(
            id="file:%s" % relative,
            node_type=GraphNodeType.FILE,
            name=file_obj.name,
            path=relative,
            qualname=relative,
        )
        module_node = GraphNode(
            id="module:%s" % module_name,
            node_type=GraphNodeType.MODULE,
            name=module_name.split(".")[-1] or module_name,
            path=relative,
            qualname=module_name,
            metadata={"module_name": module_name},
        )
        parsed = ParsedPythonFile(file_node=file_node, module_node=module_node)
        parsed.nodes.extend([file_node, module_node])
        parsed.edges.append(
            GraphEdge(
                source_id=file_node.id,
                target_id=module_node.id,
                edge_type=GraphEdgeType.DEFINES,
                metadata={"kind": "module"},
            )
        )

        tree = ast.parse(file_obj.read_text(encoding="utf-8"), filename=str(file_obj))
        visitor = _GraphVisitor(root=root, file_path=file_obj, module_node=module_node, is_test_file=is_test_file)
        visitor.visit(tree)
        parsed.nodes.extend(visitor.nodes)
        parsed.edges.extend(visitor.edges)
        parsed.imports.extend(visitor.imports)
        parsed.calls_by_node.update(visitor.calls_by_node)

        for import_name in visitor.imports:
            parsed.edges.append(
                GraphEdge(
                    source_id=file_node.id,
                    target_id="external:%s" % import_name,
                    edge_type=GraphEdgeType.IMPORTS,
                    metadata={"module": import_name},
                )
            )
        return parsed


class _GraphVisitor(ast.NodeVisitor):
    def __init__(self, root: Path, file_path: Path, module_node: GraphNode, is_test_file: bool) -> None:
        self.root = root
        self.file_path = file_path
        self.module_node = module_node
        self.is_test_file = is_test_file
        self.nodes: List[GraphNode] = []
        self.edges: List[GraphEdge] = []
        self.imports: List[str] = []
        self.calls_by_node: Dict[str, List[str]] = {}
        self._scope_stack: List[GraphNode] = []

    def visit_Import(self, node: ast.Import) -> None:
        for alias in node.names:
            self.imports.append(normalize_import_name(alias.name))

    def visit_ImportFrom(self, node: ast.ImportFrom) -> None:
        if node.module:
            self.imports.append(normalize_import_name(node.module))

    def visit_ClassDef(self, node: ast.ClassDef) -> None:
        class_node = GraphNode(
            id="class:%s:%s" % (self.file_path.relative_to(self.root).as_posix(), node.name),
            node_type=GraphNodeType.CLASS,
            name=node.name,
            path=self.file_path.relative_to(self.root).as_posix(),
            qualname="%s.%s" % (self.module_node.qualname, node.name),
            line=node.lineno,
        )
        parent = self._scope_stack[-1] if self._scope_stack else self.module_node
        self.nodes.append(class_node)
        self.edges.append(
            GraphEdge(
                source_id=parent.id,
                target_id=class_node.id,
                edge_type=GraphEdgeType.DEFINES,
                metadata={"kind": "class"},
            )
        )
        self._scope_stack.append(class_node)
        self.generic_visit(node)
        self._scope_stack.pop()

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        self._visit_function(node)

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
        self._visit_function(node)

    def _visit_function(self, node: ast.FunctionDef | ast.AsyncFunctionDef) -> None:
        parent = self._scope_stack[-1] if self._scope_stack else self.module_node
        qualname = "%s.%s" % (parent.qualname, node.name) if parent.node_type == GraphNodeType.CLASS else "%s.%s" % (self.module_node.qualname, node.name)
        node_type = GraphNodeType.TEST if (self.is_test_file or node.name.startswith("test_")) else GraphNodeType.FUNCTION
        function_node = GraphNode(
            id="%s:%s" % (node_type.value, qualname),
            node_type=node_type,
            name=node.name,
            path=self.file_path.relative_to(self.root).as_posix(),
            qualname=qualname,
            line=node.lineno,
        )
        self.nodes.append(function_node)
        self.edges.append(
            GraphEdge(
                source_id=parent.id,
                target_id=function_node.id,
                edge_type=GraphEdgeType.DEFINES,
                metadata={"kind": "function"},
            )
        )
        self._scope_stack.append(function_node)
        call_collector = _CallCollector()
        for child in node.body:
            call_collector.visit(child)
        self.calls_by_node[function_node.id] = call_collector.calls
        self.generic_visit(node)
        self._scope_stack.pop()


class _CallCollector(ast.NodeVisitor):
    def __init__(self) -> None:
        self.calls: List[str] = []

    def visit_Call(self, node: ast.Call) -> None:
        name = self._resolve_name(node.func)
        if name:
            self.calls.append(name)
        self.generic_visit(node)

    def _resolve_name(self, node: ast.AST) -> str:
        if isinstance(node, ast.Name):
            return node.id
        if isinstance(node, ast.Attribute):
            parent = self._resolve_name(node.value)
            return "%s.%s" % (parent, node.attr) if parent else node.attr
        return ""
