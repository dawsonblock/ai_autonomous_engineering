from __future__ import annotations

from collections import Counter, defaultdict
from typing import Iterable

from aae.contracts.graph import CoverageAssociation, GraphSnapshot, SymbolDefinition, SymbolReference


class ReferenceIndex:
    def __init__(
        self,
        definitions: list[SymbolDefinition],
        references: list[SymbolReference],
        coverage: list[CoverageAssociation] | None = None,
    ) -> None:
        self.definitions = definitions
        self.references = references
        self.coverage = coverage or []
        self._definitions_by_key = defaultdict(list)
        self._references_by_symbol = defaultdict(list)
        self._references_by_name = defaultdict(list)
        self._coverage_by_symbol = defaultdict(list)

        for definition in definitions:
            for key in {
                definition.name,
                definition.qualname,
                definition.file_path,
                definition.class_scope,
                definition.signature,
                "%s:%s" % (definition.file_path, definition.name),
            }:
                if key:
                    self._definitions_by_key[key].append(definition)
        for reference in references:
            if reference.resolved_symbol_id:
                self._references_by_symbol[reference.resolved_symbol_id].append(reference)
            self._references_by_name[reference.referenced_name].append(reference)
        for association in self.coverage:
            if association.target_symbol_id:
                self._coverage_by_symbol[association.target_symbol_id].append(association)

    @classmethod
    def from_snapshot(cls, snapshot: GraphSnapshot) -> "ReferenceIndex":
        return cls(snapshot.symbols, snapshot.references, snapshot.coverage)

    def lookup(self, value: str) -> list[SymbolDefinition]:
        candidates = list(self._definitions_by_key.get(value, []))
        if candidates:
            return _dedupe_definitions(candidates)
        lowered = value.lower()
        fuzzy = []
        for key, definitions in self._definitions_by_key.items():
            if lowered in key.lower():
                fuzzy.extend(definitions)
        return _dedupe_definitions(fuzzy)

    def find_references(self, symbol: str) -> list[SymbolReference]:
        definitions = self.lookup(symbol)
        references = []
        for definition in definitions:
            references.extend(self._references_by_symbol.get(definition.symbol_id, []))
        references.extend(self._references_by_name.get(symbol, []))
        return _dedupe_references(references)

    def rank_related_symbols(self, symbol: str) -> list[dict[str, object]]:
        reference_hits = Counter()
        for reference in self.find_references(symbol):
            key = reference.resolved_symbol_id or reference.referenced_name
            if not key or key == symbol:
                continue
            reference_hits[key] += 1
        ranked = []
        for key, count in reference_hits.most_common():
            definition = next((item for item in self.definitions if item.symbol_id == key), None)
            ranked.append(
                {
                    "symbol_id": getattr(definition, "symbol_id", key),
                    "name": getattr(definition, "name", key),
                    "qualname": getattr(definition, "qualname", key),
                    "file_path": getattr(definition, "file_path", ""),
                    "reference_count": count,
                    "coverage_count": len(self._coverage_by_symbol.get(getattr(definition, "symbol_id", ""), [])),
                }
            )
        return ranked

    def reference_density(self, symbol: str) -> int:
        return len(self.find_references(symbol))

    def coverage_hits(self, symbol: str) -> list[CoverageAssociation]:
        definitions = self.lookup(symbol)
        hits = []
        for definition in definitions:
            hits.extend(self._coverage_by_symbol.get(definition.symbol_id, []))
        return hits

    def symbols_for_file(self, file_path: str) -> list[SymbolDefinition]:
        return [definition for definition in self.definitions if definition.file_path == file_path]

    def related_files(self, symbols: Iterable[str]) -> list[str]:
        paths = []
        seen = set()
        for symbol in symbols:
            for definition in self.lookup(symbol):
                if definition.file_path and definition.file_path not in seen:
                    seen.add(definition.file_path)
                    paths.append(definition.file_path)
            for reference in self.find_references(symbol):
                if reference.file_path and reference.file_path not in seen:
                    seen.add(reference.file_path)
                    paths.append(reference.file_path)
        return paths


def _dedupe_definitions(definitions: list[SymbolDefinition]) -> list[SymbolDefinition]:
    deduped = []
    seen = set()
    for definition in definitions:
        if definition.symbol_id in seen:
            continue
        seen.add(definition.symbol_id)
        deduped.append(definition)
    return deduped


def _dedupe_references(references: list[SymbolReference]) -> list[SymbolReference]:
    deduped = []
    seen = set()
    for reference in references:
        key = (
            reference.source_symbol_id,
            reference.referenced_name,
            reference.resolved_symbol_id,
            reference.file_path,
            reference.line,
            reference.reference_type,
        )
        if key in seen:
            continue
        seen.add(key)
        deduped.append(reference)
    return deduped
