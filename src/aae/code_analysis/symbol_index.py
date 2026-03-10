from __future__ import annotations

from collections import defaultdict

from aae.contracts.graph import GraphSnapshot, SymbolDefinition


class SymbolIndex:
    def __init__(self, definitions: list[SymbolDefinition]) -> None:
        self._by_key = defaultdict(list)
        for definition in definitions:
            for key in [
                definition.name,
                definition.qualname,
                definition.file_path,
                definition.class_scope,
                definition.signature,
                "%s:%s" % (definition.file_path, definition.name),
            ]:
                if key:
                    self._by_key[key].append(definition)

    @classmethod
    def from_snapshot(cls, snapshot: GraphSnapshot) -> "SymbolIndex":
        return cls(snapshot.symbols)

    def lookup(self, value: str) -> list[SymbolDefinition]:
        candidates = list(self._by_key.get(value, []))
        if candidates:
            return candidates
        lowered = value.lower()
        matches = []
        for key, definitions in self._by_key.items():
            if lowered in key.lower():
                matches.extend(definitions)
        seen = set()
        deduped = []
        for definition in matches:
            if definition.symbol_id in seen:
                continue
            seen.add(definition.symbol_id)
            deduped.append(definition)
        return deduped
