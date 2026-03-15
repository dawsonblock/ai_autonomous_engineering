import pytest

from aae.agents.planner_agent import PlannerAgent
from aae.agents.engineering_agent import EngineeringAgent
from aae.agents.critic_agent import CriticAgent
from aae.agents.research_agent import ResearchAgent
from aae.agents.research_loop import ResearchLoop


@pytest.mark.anyio
async def test_planner_agent_creates_plan():
    agent = PlannerAgent()
    result = await agent.run(
        task={"goal": "fix failing test"},
        context={},
    )

    assert result["goal"] == "fix failing test"
    assert result["step_count"] > 0
    assert len(result["steps"]) > 0


@pytest.mark.anyio
async def test_engineering_agent_fix_bug():
    agent = EngineeringAgent()
    result = await agent.run(
        task={"action": "fix_bug", "bug_info": {"file": "utils.py"}},
        context={"repo_path": "/tmp/repo"},
    )

    assert result["status"] == "fix_proposed"


@pytest.mark.anyio
async def test_engineering_agent_apply_patch():
    agent = EngineeringAgent()
    result = await agent.run(
        task={"action": "apply_patch", "patch": "diff content"},
        context={},
    )

    assert result["status"] == "patch_applied"


@pytest.mark.anyio
async def test_critic_agent_evaluates():
    agent = CriticAgent()
    result = await agent.run(
        task={"artifacts": {"tests_passed": True, "lint_clean": True}},
        context={},
    )

    assert result["score"] > 0
    assert "passed" in result


@pytest.mark.anyio
async def test_research_agent_extracts_claims():
    agent = ResearchAgent()
    result = await agent.run(
        task={
            "action": "extract_claims",
            "claims": ["Method X improves accuracy", "Algorithm Y is faster"],
            "source": "paper_1",
        },
        context={},
    )

    assert result["status"] == "claims_extracted"
    assert result["count"] == 2


@pytest.mark.anyio
async def test_research_agent_gathers_evidence():
    agent = ResearchAgent()
    claim_result = await agent.run(
        task={"action": "extract_claims", "claims": ["Test claim"], "source": "test"},
        context={},
    )
    claim_id = claim_result["claim_ids"][0]

    evidence_result = await agent.run(
        task={
            "action": "gather_evidence",
            "claim_id": claim_id,
            "evidence": [{"content": "Proof 1", "source": "lab", "confidence": 0.9}],
        },
        context={},
    )

    assert evidence_result["status"] == "evidence_gathered"
    assert evidence_result["count"] == 1


def test_research_loop_runs():
    loop = ResearchLoop()
    result = loop.run("Does method X improve model Y?")

    assert result["completed"]
    assert result["question"] == "Does method X improve model Y?"
    assert result["score"] > 0
