import json

import httpx
import pytest

from aae.adapters.deep_research import DeepResearchAdapter
from aae.adapters.sec_af import SecAFAdapter
from aae.adapters.swe_af import SWEAFAdapter
from aae.contracts.tasks import TaskSpec


class FakeClient:
    def __init__(self, response=None, error=None):
        self.response = response or {}
        self.error = error
        self.calls = []

    async def execute(self, target, payload, timeout_s):
        self.calls.append({"target": target, "payload": payload, "timeout_s": timeout_s})
        if self.error is not None:
            raise self.error
        return self.response


@pytest.mark.anyio
async def test_deep_research_adapter_normalizes_output():
    client = FakeClient(
        response={
            "research_package": {
                "entities": [{"name": "NVIDIA"}],
                "relationships": [{"source": "A", "target": "B"}],
                "document": {"sections": [{"title": "Summary"}]},
            },
            "metadata": {"final_quality_score": 0.8},
        }
    )
    adapter = DeepResearchAdapter(client)
    task = TaskSpec(task_id="research", task_type="research", agent_name="deep_research", payload={"query": "AI chips"})

    result = await adapter.execute(task, {})

    assert result.status.value == "succeeded"
    assert result.normalized_output["summary"]["entity_count"] == 1
    assert result.normalized_output["events"][0]["event_type"] == "research.completed"


@pytest.mark.anyio
async def test_sec_af_adapter_emits_vulnerability_and_completion_events():
    client = FakeClient(
        response={
            "findings": [
                {
                    "id": "f1",
                    "title": "SQL Injection",
                    "severity": "high",
                    "verdict": "confirmed",
                    "location": {"file_path": "app.py"},
                }
            ],
            "confirmed": 1,
            "likely": 0,
            "by_severity": {"high": 1},
        }
    )
    adapter = SecAFAdapter(client)
    task = TaskSpec(task_id="security_baseline", task_type="security_audit", agent_name="sec_af", payload={"repo_url": "repo"})

    result = await adapter.execute(task, {})

    event_types = [event["event_type"] for event in result.normalized_output["events"]]
    assert "security.vulnerability_detected" in event_types
    assert "security.audit_completed" in event_types


@pytest.mark.anyio
async def test_swe_af_adapter_builds_additional_context_and_patch_events():
    client = FakeClient(
        response={
            "success": True,
            "dag_state": {
                "completed_issues": [
                    {
                        "issue_name": "auth",
                        "files_changed": ["auth.py"],
                        "branch_name": "branch/auth",
                    }
                ],
                "failed_issues": [],
            },
            "pr_results": [{"pr_url": "https://example.com/pr/1"}],
        }
    )
    adapter = SWEAFAdapter(client)
    task = TaskSpec(task_id="swe_build", task_type="swe_build", agent_name="swe_af", payload={"goal": "Fix auth", "repo_url": "repo"})
    memory_snapshot = {
        "task_results": {
            "research": {
                "normalized_output": {"context_for_downstream": {"research": {"summary": {"entity_count": 2}}}}
            },
            "security_baseline": {
                "normalized_output": {"context_for_downstream": {"security": {"summary": {"finding_count": 1}}}}
            },
        }
    }

    result = await adapter.execute(task, memory_snapshot)

    assert result.status.value == "succeeded"
    input_payload = result.normalized_output["input_payload"]
    additional_context = json.loads(input_payload["additional_context"])
    assert additional_context["research"]["summary"]["entity_count"] == 2
    assert additional_context["security"]["summary"]["finding_count"] == 1
    event_types = [event["event_type"] for event in result.normalized_output["events"]]
    assert "swe.patch_generated" in event_types
    assert "swe.build_completed" in event_types


@pytest.mark.anyio
async def test_transport_errors_are_marked_retryable():
    request = httpx.Request("GET", "http://localhost")
    client = FakeClient(error=httpx.ConnectError("boom", request=request))
    adapter = DeepResearchAdapter(client)
    task = TaskSpec(task_id="research", task_type="research", agent_name="deep_research", payload={"query": "AI"})

    result = await adapter.execute(task, {})

    assert result.status.value == "failed"
    assert result.error.transient is True
    assert result.error.error_type == "transport"
