from __future__ import annotations

from aae.contracts.graph import CoverageAssociation, SymbolDefinition, SymbolReference
from aae.graph.symbol_index.reference_index import ReferenceIndex


class InMemorySymbolStore:
    def __init__(self, index: ReferenceIndex | None = None) -> None:
        self.index = index or ReferenceIndex([], [], [])

    def store(
        self,
        definitions: list[SymbolDefinition],
        references: list[SymbolReference],
        coverage: list[CoverageAssociation],
    ) -> ReferenceIndex:
        self.index = ReferenceIndex(definitions, references, coverage)
        return self.index
