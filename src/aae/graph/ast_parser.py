from __future__ import annotations

import ast
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List

from aae.contracts.graph import (
    GraphEdge,
    GraphEdgeType,
    GraphNode,
    GraphNodeType,
    SymbolDefinition,
    SymbolReference,
)
from aae.graph.dependency_extractor import is_python_test_path, module_name_from_path, normalize_import_name


@dataclass
class ParsedPythonFile:
    file_node: GraphNode
    module_node: GraphNode
    nodes: List[GraphNode] = field(default_factory=list)
    edges: List[GraphEdge] = field(default_factory=list)
    imports: List[Dict[str, str]] = field(default_factory=list)
    calls_by_node: Dict[str, List[str]] = field(default_factory=dict)
    symbols: List[SymbolDefinition] = field(default_factory=list)
    references: List[SymbolReference] = field(default_factory=list)
    class_bases: Dict[str, List[str]] = field(default_factory=dict)
    methods_by_class: Dict[str, List[str]] = field(default_factory=dict)
    read_write_sets: Dict[str, Dict[str, List[str]]] = field(default_factory=dict)
    param_flows: Dict[str, List[Dict[str, str]]] = field(default_factory=dict)
    test_targets: Dict[str, List[str]] = field(default_factory=dict)


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
            metadata={"module_name": module_name},
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
        visitor = _GraphVisitor(
            root=root,
            file_path=file_obj,
            module_node=module_node,
            is_test_file=is_test_file,
        )
        visitor.visit(tree)

        parsed.nodes.extend(visitor.nodes)
        parsed.edges.extend(visitor.edges)
        parsed.imports.extend(visitor.imports)
        parsed.calls_by_node.update(visitor.calls_by_node)
        parsed.symbols.extend(visitor.symbols)
        parsed.references.extend(visitor.references)
        parsed.class_bases.update(visitor.class_bases)
        parsed.methods_by_class.update(visitor.methods_by_class)
        parsed.read_write_sets.update(visitor.read_write_sets)
        parsed.param_flows.update(visitor.param_flows)
        parsed.test_targets.update(visitor.test_targets)

        for binding in visitor.imports:
            parsed.edges.append(
                GraphEdge(
                    source_id=file_node.id,
                    target_id="external:%s" % binding["module"],
                    edge_type=GraphEdgeType.IMPORTS,
                    metadata=binding,
                )
            )
        return parsed


class _GraphVisitor(ast.NodeVisitor):
    def __init__(self, root: Path, file_path: Path, module_node: GraphNode, is_test_file: bool) -> None:
        self.root = root
        self.file_path = file_path
        self.relative_path = file_path.relative_to(root).as_posix()
        self.module_node = module_node
        self.is_test_file = is_test_file
        self.nodes: List[GraphNode] = []
        self.edges: List[GraphEdge] = []
        self.imports: List[Dict[str, str]] = []
        self.calls_by_node: Dict[str, List[str]] = {}
        self.symbols: List[SymbolDefinition] = []
        self.references: List[SymbolReference] = []
        self.class_bases: Dict[str, List[str]] = {}
        self.methods_by_class: Dict[str, List[str]] = {}
        self.read_write_sets: Dict[str, Dict[str, List[str]]] = {}
        self.param_flows: Dict[str, List[Dict[str, str]]] = {}
        self.test_targets: Dict[str, List[str]] = {}
        self._scope_stack: List[GraphNode] = []

    def visit_Import(self, node: ast.Import) -> None:
        for alias in node.names:
            binding = {
                "module": normalize_import_name(alias.name),
                "name": alias.name.split(".")[-1],
                "alias": alias.asname or alias.name.split(".")[-1],
                "kind": "import",
            }
            self.imports.append(binding)
            self.references.append(
                SymbolReference(
                    referenced_name=binding["alias"],
                    file_path=self.relative_path,
                    line=node.lineno,
                    reference_type="import",
                    metadata=binding,
                )
            )

    def visit_ImportFrom(self, node: ast.ImportFrom) -> None:
        module = normalize_import_name(node.module or "")
        for alias in node.names:
            binding = {
                "module": module,
                "name": alias.name,
                "alias": alias.asname or alias.name,
                "kind": "from",
            }
            self.imports.append(binding)
            self.references.append(
                SymbolReference(
                    referenced_name=binding["alias"],
                    file_path=self.relative_path,
                    line=node.lineno,
                    reference_type="import",
                    metadata=binding,
                )
            )

    def visit_ClassDef(self, node: ast.ClassDef) -> None:
        class_qualname = "%s.%s" % (self.module_node.qualname, node.name)
        class_node = GraphNode(
            id="class:%s" % class_qualname,
            node_type=GraphNodeType.CLASS,
            name=node.name,
            path=self.relative_path,
            qualname=class_qualname,
            line=node.lineno,
        )
        parent = self._scope_stack[-1] if self._scope_stack else self.module_node
        self.nodes.append(class_node)
        self.symbols.append(
            SymbolDefinition(
                symbol_id=class_node.id,
                name=node.name,
                qualname=class_qualname,
                symbol_type="class",
                file_path=self.relative_path,
                line=node.lineno,
            )
        )
        self.edges.append(
            GraphEdge(
                source_id=parent.id,
                target_id=class_node.id,
                edge_type=GraphEdgeType.DEFINES,
                metadata={"kind": "class"},
            )
        )
        self.class_bases[class_node.id] = [self._resolve_name(base) for base in node.bases if self._resolve_name(base)]
        self.methods_by_class[class_node.id] = []
        self._scope_stack.append(class_node)
        self.generic_visit(node)
        self._scope_stack.pop()

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        self._visit_function(node)

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
        self._visit_function(node)

    def _visit_function(self, node: ast.FunctionDef | ast.AsyncFunctionDef) -> None:
        parent = self._scope_stack[-1] if self._scope_stack else self.module_node
        if parent.node_type == GraphNodeType.CLASS:
            qualname = "%s.%s" % (parent.qualname, node.name)
        else:
            qualname = "%s.%s" % (self.module_node.qualname, node.name)
        node_type = GraphNodeType.TEST if (self.is_test_file or node.name.startswith("test_")) else GraphNodeType.FUNCTION
        function_node = GraphNode(
            id="%s:%s" % (node_type.value, qualname),
            node_type=node_type,
            name=node.name,
            path=self.relative_path,
            qualname=qualname,
            line=node.lineno,
            metadata={"signature": self._signature(node)},
        )
        self.nodes.append(function_node)
        self.symbols.append(
            SymbolDefinition(
                symbol_id=function_node.id,
                name=node.name,
                qualname=qualname,
                symbol_type=node_type.value,
                file_path=self.relative_path,
                line=node.lineno,
                class_scope=parent.qualname if parent.node_type == GraphNodeType.CLASS else "",
                signature=self._signature(node),
                metadata={"parameters": [arg.arg for arg in node.args.args]},
            )
        )
        if parent.node_type == GraphNodeType.CLASS:
            self.methods_by_class.setdefault(parent.id, []).append(node.name)
        self.edges.append(
            GraphEdge(
                source_id=parent.id,
                target_id=function_node.id,
                edge_type=GraphEdgeType.DEFINES,
                metadata={"kind": "function"},
            )
        )
        self._scope_stack.append(function_node)
        analyzer = _FunctionAnalyzer(parameters=[arg.arg for arg in node.args.args])
        for child in node.body:
            analyzer.visit(child)
        self.calls_by_node[function_node.id] = analyzer.calls
        self.read_write_sets[function_node.id] = {"reads": analyzer.reads, "writes": analyzer.writes}
        self.param_flows[function_node.id] = analyzer.param_flows
        if node_type == GraphNodeType.TEST:
            self.test_targets[function_node.id] = analyzer.calls
        for reference in analyzer.references:
            self.references.append(
                SymbolReference(
                    source_symbol_id=function_node.id,
                    referenced_name=reference["name"],
                    file_path=self.relative_path,
                    line=reference["line"],
                    reference_type=reference["kind"],
                    metadata={"scope": qualname},
                )
            )
        self.generic_visit(node)
        self._scope_stack.pop()

    def _signature(self, node: ast.FunctionDef | ast.AsyncFunctionDef) -> str:
        args = [arg.arg for arg in node.args.args]
        return "(%s)" % ", ".join(args)

    def _resolve_name(self, node: ast.AST) -> str:
        if isinstance(node, ast.Name):
            return node.id
        if isinstance(node, ast.Attribute):
            parent = self._resolve_name(node.value)
            return "%s.%s" % (parent, node.attr) if parent else node.attr
        if isinstance(node, ast.Subscript):
            return self._resolve_name(node.value)
        return ""


class _FunctionAnalyzer(ast.NodeVisitor):
    def __init__(self, parameters: List[str]) -> None:
        self.parameters = set(parameters)
        self.calls: List[str] = []
        self.reads: List[str] = []
        self.writes: List[str] = []
        self.param_flows: List[Dict[str, str]] = []
        self.references: List[Dict[str, str]] = []

    def visit_Call(self, node: ast.Call) -> None:
        name = self._resolve_name(node.func)
        if name:
            self.calls.append(name)
            self.references.append({"name": name, "line": node.lineno, "kind": "call"})
        for arg in node.args:
            param_name = self._resolve_name(arg)
            if param_name in self.parameters:
                self.param_flows.append({"param": param_name, "target": name, "line": str(node.lineno)})
        self.generic_visit(node)

    def visit_Name(self, node: ast.Name) -> None:
        if isinstance(node.ctx, ast.Load):
            self.reads.append(node.id)
            self.references.append({"name": node.id, "line": node.lineno, "kind": "read"})
        elif isinstance(node.ctx, ast.Store):
            self.writes.append(node.id)
            self.references.append({"name": node.id, "line": node.lineno, "kind": "write"})
        self.generic_visit(node)

    def _resolve_name(self, node: ast.AST) -> str:
        if isinstance(node, ast.Name):
            return node.id
        if isinstance(node, ast.Attribute):
            parent = self._resolve_name(node.value)
            return "%s.%s" % (parent, node.attr) if parent else node.attr
        if isinstance(node, ast.Constant):
            return repr(node.value)
        return ""
