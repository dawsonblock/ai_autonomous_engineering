from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent


class BugLocalizerAgent(BaseMicroAgent):
    name = "bug_localizer"

    async def run(self, task, context):
        symbol_context = context.get("symbol_context") or context.get("graph_context", {}).get("symbol_context", [])
        covering_tests = context.get("covering_tests") or context.get("graph_context", {}).get("covering_tests", [])
        candidate_files = []
        for entry in symbol_context:
            for match in entry.get("matches", []):
                score = 0.55
                if match.get("path") in covering_tests:
                    score += 0.2
                candidate_files.append(
                    {
                        "path": match.get("path", ""),
                        "reason": "symbol_and_test_overlap",
                        "score": score,
                    }
                )
        candidate_files = sorted(candidate_files, key=lambda item: item["score"], reverse=True)
        if not candidate_files:
            candidate_files = list(context.get("candidate_files", []))
        root_cause_symbol = ""
        if context.get("symbols"):
            root_cause_symbol = context["symbols"][0].get("name", "")
        return {
            "candidate_files": candidate_files[:4],
            "localized_files": candidate_files[:4],
            "root_cause_symbol": root_cause_symbol,
            "confidence": 0.72 if candidate_files else 0.35,
        }
