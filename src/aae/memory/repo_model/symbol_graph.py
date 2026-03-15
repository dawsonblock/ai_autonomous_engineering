from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict, List, Set


@dataclass
class SymbolInfo:
    name: str
    kind: str  # "class", "function", "method", "variable"
    file_path: str
    line: int = 0
    references: List[str] = field(default_factory=list)


class SymbolGraph:
    def __init__(self) -> None:
        self.symbols: Dict[str, SymbolInfo] = {}
        self._callers: Dict[str, Set[str]] = {}
        self._callees: Dict[str, Set[str]] = {}

    def add_symbol(self, symbol: SymbolInfo) -> None:
        self.symbols[symbol.name] = symbol
        self._callers.setdefault(symbol.name, set())
        self._callees.setdefault(symbol.name, set())

    def add_call(self, caller: str, callee: str) -> None:
        self._callers.setdefault(callee, set()).add(caller)
        self._callees.setdefault(caller, set()).add(callee)

    def callers_of(self, symbol_name: str) -> Set[str]:
        return self._callers.get(symbol_name, set())

    def callees_of(self, symbol_name: str) -> Set[str]:
        return self._callees.get(symbol_name, set())

    def get(self, name: str) -> SymbolInfo | None:
        return self.symbols.get(name)

    def symbols_in_file(self, file_path: str) -> List[SymbolInfo]:
        return [sym for sym in self.symbols.values() if sym.file_path == file_path]

    def impacted_by(self, changed_symbol: str) -> Set[str]:
        visited: Set[str] = set()
        stack = [changed_symbol]
        while stack:
            current = stack.pop()
            if current in visited:
                continue
            visited.add(current)
            stack.extend(self._callers.get(current, set()))
        visited.discard(changed_symbol)
        return visited
