from __future__ import annotations

from typing import Any, Dict
from uuid import uuid4

from aae.contracts.tasks import TaskSpec
from aae.contracts.workflow import WorkflowSpec


def research_only(query: str, workflow_id: str | None = None) -> WorkflowSpec:
    return WorkflowSpec(
        workflow_id=workflow_id or _workflow_id("research_only"),
        workflow_type="research_only",
        tasks=[
            TaskSpec(
                task_id="research",
                task_type="research",
                agent_name="deep_research",
                payload={"query": query},
                priority=10,
            )
        ],
        metadata={"query": query},
    )


def security_only(repo_url: str, workflow_id: str | None = None, **extra: Any) -> WorkflowSpec:
    payload = {"repo_url": repo_url}
    payload.update(extra)
    return WorkflowSpec(
        workflow_id=workflow_id or _workflow_id("security_only"),
        workflow_type="security_only",
        tasks=[
            TaskSpec(
                task_id="security_baseline",
                task_type="security_audit",
                agent_name="sec_af",
                payload=payload,
                priority=10,
            )
        ],
        metadata={"repo_url": repo_url},
    )


def swe_only(
    goal: str,
    repo_url: str,
    workflow_id: str | None = None,
    config: Dict[str, Any] | None = None,
) -> WorkflowSpec:
    payload = {"goal": goal, "repo_url": repo_url}
    if config:
        payload["config"] = config
    return WorkflowSpec(
        workflow_id=workflow_id or _workflow_id("swe_only"),
        workflow_type="swe_only",
        tasks=[
            TaskSpec(
                task_id="swe_build",
                task_type="swe_build",
                agent_name="swe_af",
                payload=payload,
                priority=10,
            )
        ],
        metadata={"goal": goal, "repo_url": repo_url},
    )


def secure_build(
    goal: str,
    repo_url: str,
    query: str | None = None,
    include_research: bool = True,
    include_post_audit: bool = False,
    workflow_id: str | None = None,
    swe_config: Dict[str, Any] | None = None,
) -> WorkflowSpec:
    tasks = []
    if include_research and query:
        tasks.append(
            TaskSpec(
                task_id="research",
                task_type="research",
                agent_name="deep_research",
                payload={"query": query},
                priority=20,
            )
        )

    tasks.append(
        TaskSpec(
            task_id="security_baseline",
            task_type="security_audit",
            agent_name="sec_af",
            payload={"repo_url": repo_url},
            priority=15,
        )
    )

    swe_depends = ["security_baseline"]
    soft_dependencies = []
    if include_research and query:
        swe_depends.append("research")
        soft_dependencies.append("research")

    swe_payload = {"goal": goal, "repo_url": repo_url}
    if swe_config:
        swe_payload["config"] = swe_config

    tasks.append(
        TaskSpec(
            task_id="swe_build",
            task_type="swe_build",
            agent_name="swe_af",
            payload=swe_payload,
            depends_on=swe_depends,
            soft_dependencies=soft_dependencies,
            priority=10,
        )
    )

    if include_post_audit:
        post_depends = ["swe_build"]
        post_soft = []
        if include_research and query:
            post_depends.append("research")
            post_soft.append("research")
        tasks.append(
            TaskSpec(
                task_id="security_post",
                task_type="security_audit",
                agent_name="sec_af",
                payload={"repo_url": repo_url},
                depends_on=post_depends,
                soft_dependencies=post_soft,
                priority=5,
            )
        )

    return WorkflowSpec(
        workflow_id=workflow_id or _workflow_id("secure_build"),
        workflow_type="secure_build",
        tasks=tasks,
        metadata={
            "goal": goal,
            "repo_url": repo_url,
            "query": query,
            "include_research": include_research,
            "include_post_audit": include_post_audit,
        },
    )


def _workflow_id(prefix: str) -> str:
    return "%s_%s" % (prefix, uuid4().hex[:8])
