from __future__ import annotations

import json
from typing import Any, Dict, List

from aae.adapters.agentfield_client import AgentFieldClient
from aae.adapters.base import AgentAdapter, now_utc
from aae.contracts.tasks import TaskSpec


class SWEAFAdapter(AgentAdapter):
    name = "swe_af"
    supported_task_types = ["swe_build"]

    def __init__(
        self,
        client: AgentFieldClient,
        target: str = "swe-planner.build",
    ) -> None:
        self.client = client
        self.target = target

    async def execute(self, task: TaskSpec, memory_snapshot: Dict[str, Any]):
        started_at = now_utc()
        attempt = _attempt_from_memory(memory_snapshot, task.task_id) + 1
        payload = dict(task.payload)
        payload["additional_context"] = build_additional_context(memory_snapshot)
        try:
            raw = await self.client.execute(self.target, payload, task.timeout_s)
            normalized = normalize_swe_output(raw)
            normalized["input_payload"] = payload
            return self._success(task, attempt, raw, normalized, started_at)
        except Exception as exc:
            return self._failure(task, attempt, exc, started_at)


def build_additional_context(memory_snapshot: Dict[str, Any]) -> str:
    task_results = memory_snapshot.get("task_results", {})
    research = ((task_results.get("research") or {}).get("normalized_output") or {}).get(
        "context_for_downstream", {}
    )
    security = ((task_results.get("security_baseline") or {}).get("normalized_output") or {}).get(
        "context_for_downstream", {}
    )
    structured = {
        "research": research.get("research", {}),
        "security": security.get("security", {}),
    }
    return json.dumps(structured, indent=2, sort_keys=True)


def normalize_swe_output(raw: Dict[str, Any]) -> Dict[str, Any]:
    dag_state = raw.get("dag_state", {})
    completed = dag_state.get("completed_issues", [])
    failed = dag_state.get("failed_issues", [])
    pr_results = raw.get("pr_results", [])

    changed_files = []
    patch_events = []
    for issue in completed:
        files = issue.get("files_changed", []) or []
        changed_files.extend(files)
        if files:
            patch_events.append(
                {
                    "event_type": "swe.patch_generated",
                    "source": "swe_af_adapter",
                    "payload": {
                        "issue_name": issue.get("issue_name"),
                        "files_changed": files,
                        "branch_name": issue.get("branch_name"),
                    },
                }
            )

    failure_events = []
    for issue in failed:
        failure_events.append(
            {
                "event_type": "swe.test_failed",
                "source": "swe_af_adapter",
                "payload": {
                    "issue_name": issue.get("issue_name"),
                    "error_message": issue.get("error_message"),
                },
            }
        )

    summary = {
        "success": bool(raw.get("success")),
        "completed_issues": len(completed),
        "failed_issues": len(failed),
        "changed_files": sorted(set(changed_files)),
        "pr_urls": [pr.get("pr_url") for pr in pr_results if pr.get("pr_url")],
    }
    events = patch_events + failure_events + [
        {
            "event_type": "swe.build_completed",
            "source": "swe_af_adapter",
            "payload": summary,
        }
    ]
    return {
        "summary": summary,
        "completed_issues": completed,
        "failed_issues": failed,
        "events": events,
    }


def _attempt_from_memory(memory_snapshot: Dict[str, Any], task_id: str) -> int:
    task_results = memory_snapshot.get("task_results", {})
    return int(task_results.get(task_id, {}).get("attempt", 0))
