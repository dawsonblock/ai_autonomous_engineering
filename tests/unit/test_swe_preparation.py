from pathlib import Path

import pytest

from aae.contracts.tasks import TaskSpec
from aae.events.event_bus import EventBus
from aae.events.event_logger import EventLogger
from aae.memory.in_memory import InMemoryMemoryStore
from aae.runtime.swe_preparation import RuntimeTaskPreparer


FIXTURE_REPO = Path(__file__).resolve().parents[1] / "fixtures" / "sample_py_repo"


@pytest.mark.anyio
async def test_runtime_task_preparer_materializes_workspace_and_builds_graph(tmp_path: Path):
    memory = InMemoryMemoryStore()
    event_bus = EventBus(logger=EventLogger(artifacts_dir=str(tmp_path)))
    preparer = RuntimeTaskPreparer(memory=memory, event_bus=event_bus, artifacts_dir=str(tmp_path))

    security_task = TaskSpec(
        task_id="security_baseline",
        task_type="security_audit",
        agent_name="sec_af",
        payload={"repo_url": str(FIXTURE_REPO)},
    )
    swe_task = TaskSpec(
        task_id="swe_build",
        task_type="swe_build",
        agent_name="swe_af",
        payload={"goal": "Fix authenticate token parsing", "repo_url": str(FIXTURE_REPO)},
    )

    prepared_security = await preparer.prepare("wf_prepare", security_task, {})
    prepared_swe = await preparer.prepare("wf_prepare", swe_task, memory.snapshot("workflow/wf_prepare"))
    workspace = memory.get("workflow/wf_prepare", "repo_workspace")
    graph_build = memory.get("workflow/wf_prepare", "graph_build")

    assert prepared_security.payload["repo_path"] == prepared_swe.payload["repo_path"]
    assert prepared_swe.payload["repo_path"] == workspace["repo_path"]
    assert Path(graph_build["sqlite_path"]).exists()
    assert Path(graph_build["json_path"]).exists()
    assert prepared_swe.payload["graph_context"]["candidate_symbols"]
    assert prepared_swe.payload["swarm_context"]["consensus_decision"]["selected_plan_id"]
    assert prepared_swe.payload["planner_decision"]["selected_branch_id"]
