from __future__ import annotations

import argparse
import asyncio
import json
import os
from pathlib import Path
from typing import Any, Dict

from aae.adapters.agentfield_client import AgentFieldClient
from aae.adapters.deep_research import DeepResearchAdapter
from aae.adapters.sec_af import SecAFAdapter
from aae.adapters.swe_af import SWEAFAdapter
from aae.controller.agent_registry import AgentRegistry
from aae.controller.controller import WorkflowController
from aae.controller.retry_policy import RetryPolicy
from aae.controller.task_scheduler import TaskScheduler
from aae.events.event_bus import EventBus
from aae.events.event_logger import EventLogger
from aae.memory.in_memory import InMemoryMemoryStore
from aae.runtime.config import SystemConfig
from aae.runtime.swe_preparation import RuntimeTaskPreparer
from aae.runtime.workflow_presets import research_only, secure_build, security_only, swe_only


def build_runtime(config_path: str) -> tuple[WorkflowController, AgentFieldClient]:
    config = SystemConfig.load(config_path)
    client = AgentFieldClient(
        base_url=config.agentfield.base_url,
        api_key=config.api_key(),
        poll_interval_s=config.agentfield.poll_interval_s,
        request_timeout_s=config.agentfield.request_timeout_s,
    )
    registry = AgentRegistry()
    registry.register(
        DeepResearchAdapter(client, target=config.siblings["af_deep_research"].node_target)
    )
    registry.register(SecAFAdapter(client, target=config.siblings["sec_af"].node_target))
    registry.register(SWEAFAdapter(client, target=config.siblings["swe_af"].node_target))

    config_root = Path(config_path).resolve().parent.parent
    artifacts_dir = str(config_root / config.controller.artifacts_dir)
    event_bus = EventBus(
        logger=EventLogger(artifacts_dir=artifacts_dir),
        redis_url=os.getenv("REDIS_URL"),
    )
    memory = InMemoryMemoryStore()
    task_preparer = RuntimeTaskPreparer(
        memory=memory,
        event_bus=event_bus,
        artifacts_dir=artifacts_dir,
    )
    controller = WorkflowController(
        registry=registry,
        memory=memory,
        event_bus=event_bus,
        scheduler=TaskScheduler(max_concurrency=config.controller.max_concurrency),
        retry_policy=RetryPolicy(),
        task_preparer=task_preparer,
    )
    return controller, client


async def run_from_args(args: argparse.Namespace) -> Dict[str, Any]:
    controller, client = build_runtime(args.config)
    await controller.event_bus.start()
    try:
        if args.workflow == "research_only":
            workflow = research_only(query=args.query)
        elif args.workflow == "security_only":
            workflow = security_only(repo_url=args.repo_url)
        elif args.workflow == "swe_only":
            workflow = swe_only(goal=args.goal, repo_url=args.repo_url)
        else:
            workflow = secure_build(
                goal=args.goal,
                repo_url=args.repo_url,
                query=args.query,
                include_research=args.include_research,
                include_post_audit=args.include_post_audit,
            )
        return await controller.run_workflow(workflow)
    finally:
        await controller.event_bus.close()
        await client.aclose()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Launch umbrella AI kernel workflows")
    parser.add_argument(
        "--config",
        default=os.getenv("AAE_CONFIG", "configs/system_config.yaml"),
    )
    parser.add_argument(
        "--workflow",
        choices=["research_only", "security_only", "swe_only", "secure_build"],
        default="secure_build",
    )
    parser.add_argument("--query", default="")
    parser.add_argument("--goal", default="")
    parser.add_argument("--repo-url", default="")
    parser.add_argument("--include-research", action="store_true", default=False)
    parser.add_argument("--include-post-audit", action="store_true", default=False)
    args = parser.parse_args()
    validate_args(parser, args)
    return args


def validate_args(parser: argparse.ArgumentParser, args: argparse.Namespace) -> None:
    if args.workflow == "research_only" and not args.query.strip():
        parser.error("--query is required for research_only")
    if args.workflow == "security_only" and not args.repo_url.strip():
        parser.error("--repo-url is required for security_only")
    if args.workflow == "swe_only":
        if not args.goal.strip():
            parser.error("--goal is required for swe_only")
        if not args.repo_url.strip():
            parser.error("--repo-url is required for swe_only")
    if args.workflow == "secure_build":
        if not args.goal.strip():
            parser.error("--goal is required for secure_build")
        if not args.repo_url.strip():
            parser.error("--repo-url is required for secure_build")
        if args.include_research and not args.query.strip():
            parser.error("--query is required when --include-research is set")


def main() -> None:
    args = parse_args()
    result = asyncio.run(run_from_args(args))
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
