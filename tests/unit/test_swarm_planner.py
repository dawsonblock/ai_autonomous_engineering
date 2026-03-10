from pathlib import Path

import pytest

from aae.agents.micro_agents.orchestration.swarm_controller import SwarmController
from aae.graph.graph_query import GraphQueryEngine
from aae.graph.repo_graph_builder import RepoGraphBuilder
from aae.planner.planner_runtime import PlannerRuntime
from aae.tools.graph_tools import GraphContextBuilder


FIXTURE_REPO = Path(__file__).resolve().parents[1] / "fixtures" / "sample_py_repo"


@pytest.mark.anyio
async def test_swarm_controller_runs_consensus_and_simulation(tmp_path: Path):
    build = RepoGraphBuilder().build(
        repo_path=str(FIXTURE_REPO),
        sqlite_path=str(tmp_path / "graph.sqlite3"),
        json_path=str(tmp_path / "graph.json"),
    )
    graph = GraphQueryEngine.from_sqlite(build.sqlite_path)
    graph_context = GraphContextBuilder(graph).build("Fix authenticate token parsing")

    swarm = SwarmController()
    result = await swarm.run(
        task={"goal": "Fix authenticate token parsing"},
        context={"repo_path": str(FIXTURE_REPO), "graph": graph, "graph_context": graph_context},
    )

    assert result["shortlisted_candidates"]
    assert result["consensus_decision"]["selected_plan_id"]
    assert result["patch_candidate"]["changed_files"]
    assert "risk_score" in result["simulation"]
    assert isinstance(result["review"]["accept"], bool)

    planner = PlannerRuntime()
    decision = planner.plan(
        workflow_goal="Fix authenticate token parsing",
        graph_context=graph_context,
        memory_state={},
        swarm_result=result,
    )

    assert decision.selected_branch_id
    assert decision.branches
