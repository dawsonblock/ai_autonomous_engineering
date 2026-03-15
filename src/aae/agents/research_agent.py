from __future__ import annotations

from typing import Any, Dict

from aae.agents.micro_agents.base import BaseMicroAgent
from aae.memory.knowledge_graph import KnowledgeGraph


class ResearchAgent(BaseMicroAgent):
    name = "researcher"
    domain = "research"

    def __init__(self, knowledge_graph: KnowledgeGraph | None = None) -> None:
        self.knowledge_graph = knowledge_graph or KnowledgeGraph()

    async def run(self, task: Dict[str, Any], context: Dict[str, Any]) -> Dict[str, Any]:
        action = task.get("action", "")
        if action == "extract_claims":
            return await self.extract_claims(task, context)
        if action == "gather_evidence":
            return await self.gather_evidence(task, context)
        return {"status": "unknown_action", "action": action}

    async def extract_claims(self, task: Dict[str, Any], context: Dict[str, Any]) -> Dict[str, Any]:
        claims_text = task.get("claims", [])
        source = task.get("source", "")
        claim_ids = []
        for text in claims_text:
            claim = self.knowledge_graph.create_claim(text=text, source=source)
            claim_ids.append(claim.claim_id)
        return {"status": "claims_extracted", "claim_ids": claim_ids, "count": len(claim_ids)}

    async def gather_evidence(self, task: Dict[str, Any], context: Dict[str, Any]) -> Dict[str, Any]:
        claim_id = task.get("claim_id")
        if not claim_id:
            return {
                "status": "invalid_claim_id",
                "reason": "claim_id is required to gather evidence.",
            }
        evidence_items = task.get("evidence", [])
        evidence_ids = []
        for item in evidence_items:
            evidence = self.knowledge_graph.create_evidence(
                claim_id=claim_id,
                content=item.get("content", ""),
                source=item.get("source", ""),
                confidence=item.get("confidence", 0.0),
            )
            evidence_ids.append(evidence.evidence_id)
        return {"status": "evidence_gathered", "evidence_ids": evidence_ids, "count": len(evidence_ids)}
