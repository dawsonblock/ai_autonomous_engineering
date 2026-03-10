from __future__ import annotations

from typing import Any, Dict, List

from aae.adapters.agentfield_client import AgentFieldClient
from aae.adapters.base import AgentAdapter, now_utc
from aae.contracts.tasks import TaskSpec


class SecAFAdapter(AgentAdapter):
    name = "sec_af"
    supported_task_types = ["security_audit"]

    def __init__(
        self,
        client: AgentFieldClient,
        target: str = "sec-af.audit",
    ) -> None:
        self.client = client
        self.target = target

    async def execute(self, task: TaskSpec, memory_snapshot: Dict[str, Any]):
        started_at = now_utc()
        attempt = _attempt_from_memory(memory_snapshot, task.task_id) + 1
        try:
            raw = await self.client.execute(self.target, task.payload, task.timeout_s)
            normalized = normalize_security_output(raw)
            return self._success(task, attempt, raw, normalized, started_at)
        except Exception as exc:
            return self._failure(task, attempt, exc, started_at)


def normalize_security_output(raw: Dict[str, Any]) -> Dict[str, Any]:
    findings = raw.get("findings", [])
    if not isinstance(findings, list):
        findings = []
    summary = {
        "finding_count": len(findings),
        "confirmed": raw.get("confirmed", 0),
        "likely": raw.get("likely", 0),
        "by_severity": raw.get("by_severity", {}),
    }
    normalized_findings = []
    events = []
    for finding in findings:
        normalized_findings.append(
            {
                "id": finding.get("id"),
                "title": finding.get("title"),
                "severity": finding.get("severity"),
                "verdict": finding.get("verdict"),
                "file_path": (finding.get("location") or {}).get("file_path"),
            }
        )
        events.append(
            {
                "event_type": "security.vulnerability_detected",
                "source": "sec_af_adapter",
                "payload": normalized_findings[-1],
            }
        )
    events.append(
        {
            "event_type": "security.audit_completed",
            "source": "sec_af_adapter",
            "payload": summary,
        }
    )
    return {
        "summary": summary,
        "findings": normalized_findings,
        "context_for_downstream": {"security": {"summary": summary, "findings": normalized_findings}},
        "events": events,
    }


def _attempt_from_memory(memory_snapshot: Dict[str, Any], task_id: str) -> int:
    task_results = memory_snapshot.get("task_results", {})
    return int(task_results.get(task_id, {}).get("attempt", 0))
