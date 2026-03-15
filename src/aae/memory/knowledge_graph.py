from __future__ import annotations

import time
from typing import Any, Dict, List, Set
from uuid import uuid4


class Claim:
    __slots__ = ("claim_id", "text", "source", "timestamp", "evidence_ids", "experiment_ids", "metadata")

    def __init__(
        self,
        text: str,
        source: str = "",
        metadata: Dict[str, Any] | None = None,
    ) -> None:
        self.claim_id = uuid4().hex
        self.text = text
        self.source = source
        self.timestamp = time.time()
        self.evidence_ids: List[str] = []
        self.experiment_ids: List[str] = []
        self.metadata = metadata or {}

    def to_dict(self) -> Dict[str, Any]:
        return {
            "claim_id": self.claim_id,
            "text": self.text,
            "source": self.source,
            "timestamp": self.timestamp,
            "evidence_ids": self.evidence_ids,
            "experiment_ids": self.experiment_ids,
            "metadata": self.metadata,
        }


class Evidence:
    __slots__ = ("evidence_id", "claim_id", "content", "source", "confidence", "timestamp")

    def __init__(
        self,
        claim_id: str,
        content: str,
        source: str = "",
        confidence: float = 0.0,
    ) -> None:
        self.evidence_id = uuid4().hex
        self.claim_id = claim_id
        self.content = content
        self.source = source
        self.confidence = confidence
        self.timestamp = time.time()

    def to_dict(self) -> Dict[str, Any]:
        return {
            "evidence_id": self.evidence_id,
            "claim_id": self.claim_id,
            "content": self.content,
            "source": self.source,
            "confidence": self.confidence,
            "timestamp": self.timestamp,
        }


class KnowledgeGraph:
    def __init__(self) -> None:
        self._claims: Dict[str, Claim] = {}
        self._evidence: Dict[str, Evidence] = {}

    def add_claim(self, claim: Claim) -> str:
        self._claims[claim.claim_id] = claim
        return claim.claim_id

    def create_claim(self, text: str, source: str = "", metadata: Dict[str, Any] | None = None) -> Claim:
        claim = Claim(text=text, source=source, metadata=metadata)
        self.add_claim(claim)
        return claim

    def add_evidence(self, evidence: Evidence) -> str:
        self._evidence[evidence.evidence_id] = evidence
        claim = self._claims.get(evidence.claim_id)
        if claim:
            claim.evidence_ids.append(evidence.evidence_id)
        return evidence.evidence_id

    def create_evidence(
        self,
        claim_id: str,
        content: str,
        source: str = "",
        confidence: float = 0.0,
    ) -> Evidence:
        evidence = Evidence(
            claim_id=claim_id,
            content=content,
            source=source,
            confidence=confidence,
        )
        self.add_evidence(evidence)
        return evidence

    def get_claim(self, claim_id: str) -> Claim | None:
        return self._claims.get(claim_id)

    def evidence_for(self, claim_id: str) -> List[Evidence]:
        claim = self._claims.get(claim_id)
        if not claim:
            return []
        return [self._evidence[eid] for eid in claim.evidence_ids if eid in self._evidence]

    def all_claims(self) -> List[Claim]:
        return list(self._claims.values())

    @property
    def claim_count(self) -> int:
        return len(self._claims)

    @property
    def evidence_count(self) -> int:
        return len(self._evidence)
