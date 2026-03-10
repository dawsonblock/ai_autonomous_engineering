import json
from pathlib import Path

import pytest

from aae.adapters.agentfield_client import AgentFieldExecutionError
from aae.adapters.deep_research import DeepResearchAdapter
from aae.adapters.sec_af import SecAFAdapter
from aae.adapters.swe_af import SWEAFAdapter
from aae.controller.agent_registry import AgentRegistry
from aae.controller.controller import WorkflowController
from aae.controller.task_scheduler import TaskScheduler
from aae.events.event_bus import EventBus
from aae.events.event_logger import EventLogger
from aae.events.event_replay import EventReplay
from aae.memory.in_memory import InMemoryMemoryStore
from aae.runtime.workflow_presets import research_only, secure_build


class FakeAgentFieldClient:
    def __init__(self, handlers):
        self.handlers = handlers
        self.calls = []

    async def execute(self, target, payload, timeout_s):
        self.calls.append({"target": target, "payload": payload, "timeout_s": timeout_s})
        handler = self.handlers[target]
        if isinstance(handler, Exception):
            raise handler
        if callable(handler):
            return handler(payload)
        return handler


def build_controller(tmp_path: Path, client: FakeAgentFieldClient):
    registry = AgentRegistry()
    registry.register(DeepResearchAdapter(client))
    registry.register(SecAFAdapter(client))
    registry.register(SWEAFAdapter(client))

    event_bus = EventBus(logger=EventLogger(artifacts_dir=str(tmp_path)))
    controller = WorkflowController(
        registry=registry,
        memory=InMemoryMemoryStore(),
        event_bus=event_bus,
        scheduler=TaskScheduler(max_concurrency=4),
    )
    return controller


@pytest.mark.anyio
async def test_research_only_completes_and_writes_memory(tmp_path: Path):
    client = FakeAgentFieldClient(
        {
            "meta_deep_research.execute_deep_research": {
                "research_package": {"entities": [], "relationships": [], "document": {}},
                "metadata": {"final_quality_score": 0.9},
            }
        }
    )
    controller = build_controller(tmp_path, client)
    workflow = research_only(query="AI chips", workflow_id="wf_research")

    result = await controller.run_workflow(workflow)
    workflow_snapshot = controller.memory.snapshot("workflow/wf_research")

    assert result["final_states"]["research"] == "succeeded"
    assert "research" in workflow_snapshot["task_results"]


@pytest.mark.anyio
async def test_secure_build_injects_upstream_context_into_swe(tmp_path: Path):
    client = FakeAgentFieldClient(
        {
            "meta_deep_research.execute_deep_research": {
                "research_package": {
                    "entities": [{"name": "A"}],
                    "relationships": [],
                    "document": {},
                },
                "metadata": {"final_quality_score": 0.75},
            },
            "sec-af.audit": {
                "findings": [
                    {
                        "id": "f1",
                        "title": "auth issue",
                        "severity": "medium",
                        "verdict": "confirmed",
                        "location": {"file_path": "auth.py"},
                    }
                ],
                "confirmed": 1,
                "likely": 0,
                "by_severity": {"medium": 1},
            },
            "swe-planner.build": {
                "success": True,
                "dag_state": {
                    "completed_issues": [{"issue_name": "auth", "files_changed": ["auth.py"], "branch_name": "branch/auth"}],
                    "failed_issues": [],
                },
                "pr_results": [],
            },
        }
    )
    controller = build_controller(tmp_path, client)
    workflow = secure_build(
        goal="Fix auth",
        repo_url="https://example.com/repo.git",
        query="Find auth risks",
        include_research=True,
        include_post_audit=False,
        workflow_id="wf_secure",
    )

    result = await controller.run_workflow(workflow)

    assert result["final_states"]["research"] == "succeeded"
    assert result["final_states"]["security_baseline"] == "succeeded"
    assert result["final_states"]["swe_build"] == "succeeded"

    swe_call = next(call for call in client.calls if call["target"] == "swe-planner.build")
    context = json.loads(swe_call["payload"]["additional_context"])
    assert context["research"]["summary"]["entity_count"] == 1
    assert context["security"]["summary"]["finding_count"] == 1


@pytest.mark.anyio
async def test_hard_dependency_failure_blocks_downstream(tmp_path: Path):
    client = FakeAgentFieldClient(
        {
            "sec-af.audit": AgentFieldExecutionError("security audit failed"),
            "swe-planner.build": {
                "success": True,
                "dag_state": {"completed_issues": [], "failed_issues": []},
                "pr_results": [],
            },
        }
    )
    controller = build_controller(tmp_path, client)
    workflow = secure_build(
        goal="Fix auth",
        repo_url="https://example.com/repo.git",
        query=None,
        include_research=False,
        include_post_audit=False,
        workflow_id="wf_blocked",
    )

    result = await controller.run_workflow(workflow)

    assert result["final_states"]["security_baseline"] == "failed"
    assert result["final_states"]["swe_build"] == "blocked"


@pytest.mark.anyio
async def test_optional_post_audit_is_absent_when_disabled(tmp_path: Path):
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
        goal="Fix auth",
        repo_url="https://example.com/repo.git",
        include_research=False,
        include_post_audit=False,
        workflow_id="wf_no_post",
    )

    result = await controller.run_workflow(workflow)

    assert "security_post" not in result["final_states"]


@pytest.mark.anyio
async def test_in_memory_event_bus_and_replay_reconstruct_observable_history(tmp_path: Path):
    client = FakeAgentFieldClient(
        {
            "meta_deep_research.execute_deep_research": {
                "research_package": {"entities": [], "relationships": [], "document": {}},
                "metadata": {"final_quality_score": 0.9},
            }
        }
    )
    controller = build_controller(tmp_path, client)
    assert controller.event_bus.transport_mode == "memory"

    workflow = research_only(query="AI chips", workflow_id="wf_replay")
    await controller.run_workflow(workflow)

    replay_bus = EventBus()
    observed = []

    async def listener(event):
        observed.append(event.event_type)

    replay_bus.subscribe("*", listener)
    replay = EventReplay(replay_bus)
    await replay.replay(tmp_path / "events" / "wf_replay.jsonl")

    assert observed[0] == "workflow.started"
    assert "task.succeeded" in observed
    assert observed[-1] == "workflow.completed"
