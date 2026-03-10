from __future__ import annotations

from typing import Dict, Iterable, List

from aae.contracts.graph import SymbolDefinition, SymbolReference
from aae.graph.ast_parser import ParsedPythonFile


class SymbolTableBuilder:
    def build(self, parsed_files: Iterable[ParsedPythonFile]) -> tuple[list[SymbolDefinition], list[SymbolReference], dict]:
        definitions: List[SymbolDefinition] = []
        references: List[SymbolReference] = []
        index: Dict[str, list[SymbolDefinition]] = {}
        for parsed in parsed_files:
            definitions.extend(parsed.symbols)
            references.extend(parsed.references)
        for definition in definitions:
            for key in [definition.name, definition.qualname, definition.signature]:
                if not key:
                    continue
                index.setdefault(key, []).append(definition)
        return definitions, references, index
