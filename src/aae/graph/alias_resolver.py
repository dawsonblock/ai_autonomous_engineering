from __future__ import annotations

from typing import Dict, Iterable, List

from aae.contracts.graph import SymbolDefinition, SymbolReference
from aae.graph.ast_parser import ParsedPythonFile


class AliasResolver:
    def resolve(
        self,
        parsed_files: Iterable[ParsedPythonFile],
        definitions: list[SymbolDefinition],
        references: list[SymbolReference],
    ) -> list[SymbolReference]:
        by_name: Dict[str, list[SymbolDefinition]] = {}
        for definition in definitions:
            by_name.setdefault(definition.name, []).append(definition)
            by_name.setdefault(definition.qualname, []).append(definition)

        import_aliases: Dict[tuple[str, str], str] = {}
        for parsed in parsed_files:
            for binding in parsed.imports:
                resolved_name = ("%s.%s" % (binding["module"], binding["name"])).strip(".")
                import_aliases[(parsed.file_node.path, binding["alias"])] = resolved_name

        resolved: List[SymbolReference] = []
        for reference in references:
            key = (reference.file_path, reference.referenced_name)
            resolved_name = import_aliases.get(key, reference.referenced_name)
            candidates = by_name.get(resolved_name, []) or by_name.get(resolved_name.split(".")[-1], [])
            resolved.append(
                reference.model_copy(
                    update={
                        "resolved_symbol_id": candidates[0].symbol_id if candidates else reference.resolved_symbol_id,
                        "metadata": {**reference.metadata, "resolved_name": resolved_name},
                    }
                )
            )
        return resolved
