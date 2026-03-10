from __future__ import annotations

from aae.contracts.graph import GraphSnapshot, SymbolDefinition
from aae.graph.symbol_index.reference_index import ReferenceIndex


class SymbolIndex:
    def __init__(self, definitions: list[SymbolDefinition], reference_index: ReferenceIndex | None = None) -> None:
        self.reference_index = reference_index or ReferenceIndex(definitions, [], [])
        self._definitions = definitions

    @classmethod
    def from_snapshot(cls, snapshot: GraphSnapshot) -> "SymbolIndex":
        return cls(snapshot.symbols, reference_index=ReferenceIndex.from_snapshot(snapshot))

    def lookup(self, value: str) -> list[SymbolDefinition]:
        return self.reference_index.lookup(value)

    def find_references(self, symbol: str):
        return self.reference_index.find_references(symbol)

    def rank_related_symbols(self, symbol: str):
        return self.reference_index.rank_related_symbols(symbol)
