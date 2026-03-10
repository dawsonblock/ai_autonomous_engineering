from __future__ import annotations

from aae.contracts.graph import GraphSnapshot


class CallSignatureResolver:
    def resolve(self, snapshot: GraphSnapshot, qualname: str) -> dict[str, str]:
        resolved = {}
        for symbol in snapshot.symbols:
            if symbol.qualname == qualname:
                resolved["signature"] = symbol.signature
                resolved["symbol_id"] = symbol.symbol_id
                break
        resolved["resolved_calls"] = [
            reference.metadata.get("resolved_name", reference.referenced_name)
            for reference in snapshot.references
            if reference.source_symbol_id == resolved.get("symbol_id") and reference.reference_type == "call"
        ]
        return resolved
