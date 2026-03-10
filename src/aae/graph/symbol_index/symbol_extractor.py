from __future__ import annotations

from aae.contracts.graph import CoverageAssociation, GraphSnapshot, SymbolDefinition, SymbolReference


class SymbolExtractor:
    def extract(self, snapshot: GraphSnapshot) -> tuple[list[SymbolDefinition], list[SymbolReference], list[CoverageAssociation]]:
        return list(snapshot.symbols), list(snapshot.references), list(snapshot.coverage)
