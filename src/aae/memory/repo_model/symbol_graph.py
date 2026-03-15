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
        # Internal maps are keyed by a unique symbol key, not just the bare name.
        # The key format is "<file_path>:<name>".
        self.symbols: Dict[str, SymbolInfo] = {}
        self._callers: Dict[str, Set[str]] = {}
        self._callees: Dict[str, Set[str]] = {}
        # Secondary index to find symbols by bare name.
        self._symbols_by_name: Dict[str, Set[str]] = {}

    def _symbol_key(self, symbol: SymbolInfo) -> str:
        """Return a unique, stable key for a symbol."""
        return f"{symbol.file_path}:{symbol.name}"

    def _resolve_to_keys(self, identifier: str) -> Set[str]:
        """
        Resolve an identifier to one or more internal symbol keys.

        If the identifier matches an existing internal key, return that.
        Otherwise, treat it as a bare name and return all keys for that name.
        """
        if identifier in self.symbols:
            return {identifier}
        return set(self._symbols_by_name.get(identifier, set()))

    def add_symbol(self, symbol: SymbolInfo) -> None:
        key = self._symbol_key(symbol)
        self.symbols[key] = symbol
        self._callers.setdefault(key, set())
        self._callees.setdefault(key, set())
        self._symbols_by_name.setdefault(symbol.name, set()).add(key)

    def add_call(self, caller: str, callee: str) -> None:
        """
        Record that 'caller' calls 'callee'.

        The arguments are expected to be internal symbol keys where possible
        (i.e., '<file_path>:<name>'). If bare names are used and are not
        unique, the call graph may be ambiguous.
        """
        self._callers.setdefault(callee, set()).add(caller)
        self._callees.setdefault(caller, set()).add(callee)

    def callers_of(self, symbol_name: str) -> Set[str]:
        """
        Return all callers of the given symbol.

        'symbol_name' may be an internal key or a bare name; in the latter
        case, callers of all matching symbols are returned.
        """
        result: Set[str] = set()
        for key in self._resolve_to_keys(symbol_name):
            result.update(self._callers.get(key, set()))
        return result

    def callees_of(self, symbol_name: str) -> Set[str]:
        """
        Return all callees of the given symbol.

        'symbol_name' may be an internal key or a bare name; in the latter
        case, callees of all matching symbols are returned.
        """
        result: Set[str] = set()
        for key in self._resolve_to_keys(symbol_name):
            result.update(self._callees.get(key, set()))
        return result

    def get(self, name: str) -> SymbolInfo | None:
        """
        Retrieve a symbol by internal key or, if unique, by bare name.

        If multiple symbols share the same name, None is returned when
        looking up by bare name to avoid ambiguity.
        """
        # First try direct key lookup.
        sym = self.symbols.get(name)
        if sym is not None:
            return sym
        # Fall back to lookup by bare name.
        keys = self._symbols_by_name.get(name)
        if not keys:
            return None
        if len(keys) == 1:
            only_key = next(iter(keys))
            return self.symbols.get(only_key)
        # Ambiguous bare name.
        return None

    def symbols_in_file(self, file_path: str) -> List[SymbolInfo]:
        return [sym for sym in self.symbols.values() if sym.file_path == file_path]

    def impacted_by(self, changed_symbol: str) -> Set[str]:
        """
        Return all symbols impacted (transitively) by a changed symbol.

        'changed_symbol' may be an internal key or a bare name; in the latter
        case, all matching symbols are treated as changed.
        """
        visited: Set[str] = set()
        stack = list(self._resolve_to_keys(changed_symbol))
        while stack:
            current = stack.pop()
            if current in visited:
                continue
            visited.add(current)
            stack.extend(self._callers.get(current, set()))
        for key in self._resolve_to_keys(changed_symbol):
            visited.discard(key)
        return visited
