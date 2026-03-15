from __future__ import annotations

from typing import Any, Dict

from aae.agents.micro_agents.base import BaseMicroAgent


class EngineeringAgent(BaseMicroAgent):
    name = "engineer"
    domain = "swe"

    async def run(self, task: Dict[str, Any], context: Dict[str, Any]) -> Dict[str, Any]:
        action = task.get("action", "")
        if action == "fix_bug":
            return await self.fix_bug(task, context)
        if action == "apply_patch":
            return await self.apply_patch(task, context)
        if action == "write_code":
            return await self.write_code(task, context)
        return {"status": "unknown_action", "action": action}

    async def fix_bug(self, task: Dict[str, Any], context: Dict[str, Any]) -> Dict[str, Any]:
        repo_path = context.get("repo_path", "")
        bug_info = task.get("bug_info", {})
        return {
            "status": "fix_proposed",
            "repo_path": repo_path,
            "bug_info": bug_info,
            "patch": task.get("patch", ""),
        }

    async def apply_patch(self, task: Dict[str, Any], context: Dict[str, Any]) -> Dict[str, Any]:
        patch = task.get("patch", "")
        return {
            "status": "patch_applied",
            "patch_size": len(patch),
        }

    async def write_code(self, task: Dict[str, Any], context: Dict[str, Any]) -> Dict[str, Any]:
        spec = task.get("spec", {})
        return {
            "status": "code_written",
            "spec": spec,
        }
