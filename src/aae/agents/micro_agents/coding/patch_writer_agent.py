from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent


class PatchWriterAgent(BaseMicroAgent):
    name = "patch_writer"

    async def run(self, task, context):
        plan = context.get("selected_plan") or {}
        changed_files = plan.get("target_files", [])[:2] or ["unknown.py"]
        diff_chunks = []
        for path in changed_files:
            diff_chunks.append(
                "\n".join(
                    [
                        "diff --git a/%s b/%s" % (path, path),
                        "--- a/%s" % path,
                        "+++ b/%s" % path,
                        "@@",
                        "+# planned change: %s" % plan.get("summary", "apply selected plan"),
                    ]
                )
            )
        return {
            "plan_id": plan.get("id", ""),
            "diff": "\n".join(diff_chunks),
            "changed_files": changed_files,
            "confidence": float(plan.get("confidence", 0.0)),
        }
