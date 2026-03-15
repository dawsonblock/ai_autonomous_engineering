from aae.memory.knowledge_graph import Claim, Evidence, KnowledgeGraph


def test_knowledge_graph_add_claim():
    kg = KnowledgeGraph()
    claim = kg.create_claim(text="Method X improves accuracy", source="paper_1")

    assert kg.claim_count == 1
    retrieved = kg.get_claim(claim.claim_id)
    assert retrieved is not None
    assert retrieved.text == "Method X improves accuracy"


def test_knowledge_graph_add_evidence():
    kg = KnowledgeGraph()
    claim = kg.create_claim(text="Algorithm Y is faster")
    evidence = kg.create_evidence(
        claim_id=claim.claim_id,
        content="Benchmark shows 2x speedup",
        source="experiment_1",
        confidence=0.85,
    )

    assert kg.evidence_count == 1
    evidence_list = kg.evidence_for(claim.claim_id)
    assert len(evidence_list) == 1
    assert evidence_list[0].confidence == 0.85


def test_knowledge_graph_multiple_evidence():
    kg = KnowledgeGraph()
    claim = kg.create_claim(text="Hypothesis A")
    kg.create_evidence(claim_id=claim.claim_id, content="Evidence 1", confidence=0.7)
    kg.create_evidence(claim_id=claim.claim_id, content="Evidence 2", confidence=0.9)

    evidence = kg.evidence_for(claim.claim_id)
    assert len(evidence) == 2


def test_knowledge_graph_all_claims():
    kg = KnowledgeGraph()
    kg.create_claim(text="Claim 1")
    kg.create_claim(text="Claim 2")
    kg.create_claim(text="Claim 3")

    assert len(kg.all_claims()) == 3


def test_claim_to_dict():
    claim = Claim(text="test claim", source="test")
    data = claim.to_dict()
    assert data["text"] == "test claim"
    assert "claim_id" in data
    assert "timestamp" in data


def test_evidence_to_dict():
    evidence = Evidence(claim_id="c1", content="proof", source="lab", confidence=0.9)
    data = evidence.to_dict()
    assert data["claim_id"] == "c1"
    assert data["confidence"] == 0.9
