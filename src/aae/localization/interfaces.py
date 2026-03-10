from __future__ import annotations

from typing import Any, Dict, List, Protocol, Tuple

from .models import FailureSignal


class Ranker(Protocol):
    def rank(self, failure_signals: List[FailureSignal], context: Dict[str, Any]) -> Dict[Any, float]:
        ...

class SupportsFileRanking(Protocol):
    def rank_files(self, failure_signals: List[FailureSignal], context: Dict[str, Any]) -> Dict[str, float]:
        ...

class SupportsFunctionRanking(Protocol):
    def rank_functions(self, failure_signals: List[FailureSignal], context: Dict[str, Any]) -> Dict[Tuple[str, str], float]:
        ...

class SupportsSpanRanking(Protocol):
    def rank_spans(self, failure_signals: List[FailureSignal], context: Dict[str, Any]) -> Dict[Tuple[str, int, int], float]:
        ...
