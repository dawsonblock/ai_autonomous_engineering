from __future__ import annotations

from aae.graph.symbol_index.reference_index import ReferenceIndex


class SymbolQueryService:
    def __init__(self, index: ReferenceIndex) -> None:
        self.index = index

    def lookup(self, value: str):
        return self.index.lookup(value)

    def find_references(self, symbol: str):
        return self.index.find_references(symbol)

    def rank_related_symbols(self, symbol: str):
        return self.index.rank_related_symbols(symbol)
