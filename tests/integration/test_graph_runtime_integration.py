import json
from pathlib import Path

import pytest

from aae.adapters.deep_research import DeepResearchAdapter
from aae.adapters.sec_af import SecAFAdapter
from aae.adapters.swe_af import SWEAFAdapter
from aae.controller.agent_registry import AgentRegistry
from aae.controller.controller import WorkflowController
from aae.controller.task_scheduler import TaskScheduler
from aae.events.event_bus import EventBus
from aae.events.event_logger import EventLogger
from aae.memory.in_memory import InMemoryMemoryStore
from aae.runtime.swe_preparation import RuntimeTaskPreparer
from aae.runtime.workflow_presets import secure_build


FIXTURE_REPO = Path(__file__).resolve().parents[1] / "fixtures" / "sample_py_repo"


class FakeAgentFieldClient:
    def __init__(self, handlers):
        self.handlers = handlers
        self.calls = []

    async def execute(self, target, payload, timeout_s):
        self.calls.append({"target": target, "payload": payload, "timeout_s": timeout_s})
        handler = self.handlers[target]
        if callable(handler):
            return handler(payload)
        return handler


def build_controller(tmp_path: Path, client: FakeAgentFieldClient):
    registry = AgentRegistry()
    registry.register(DeepResearchAdapter(client))
    registry.register(SecAFAdapter(client))
    registry.register(SWEAFAdapter(client))
    memory = InMemoryMemoryStore()
    event_bus = EventBus(logger=EventLogger(artifacts_dir=str(tmp_path)))
    task_preparer = RuntimeTaskPreparer(memory=memory, event_bus=event_bus, artifacts_dir=str(tmp_path))
    return WorkflowController(
        registry=registry,
        memory=memory,
        event_bus=event_bus,
        scheduler=TaskScheduler(max_concurrency=4),
        task_preparer=task_preparer,
    )


@pytest.mark.anyio
async def test_secure_build_uses_shared_workspace_and_graph_context(tmp_path: Path):
    client = FakeAgentFieldClient(
        {
            "sec-af.audit": {"findings": [], "confirmed": 0, "likely": 0, "by_severity": {}},
            "swe-planner.build": {
                "success": True,
                "dag_state": {"completed_issues": [], "failed_issues": []},
                "pr_results": [],
            },
        }
    )
    controller = build_controller(tmp_path, client)
    workflow = secure_build(
        goal="Fix authenticate token parsing",
        repo_url=str(FIXTURE_REPO),
        include_research=False,
        include_post_audit=False,
        workflow_id="wf_graph_runtime",
    )

    result = await controller.run_workflow(workflow)

    assert result["final_states"]["security_baseline"] == "succeeded"
    assert result["final_states"]["swe_build"] == "succeeded"

    security_call = next(call for call in client.calls if call["target"] == "sec-af.audit")
    swe_call = next(call for call in client.calls if call["target"] == "swe-planner.build")
    assert security_call["payload"]["repo_path"] == swe_call["payload"]["repo_path"]

    context = json.loads(swe_call["payload"]["additional_context"])
    assert context["repo_workspace"]["repo_path"] == swe_call["payload"]["repo_path"]
    assert context["graph_context"]["call_chains"]
    assert context["planner_decision"]["selected_branch_id"]
