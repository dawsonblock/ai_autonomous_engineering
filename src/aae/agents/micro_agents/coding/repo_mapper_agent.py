from __future__ import annotations

from aae.agents.micro_agents.base import BaseMicroAgent


class RepoMapperAgent(BaseMicroAgent):
    name = "repo_mapper"

    async def run(self, task, context):
        graph_context = context.get("graph_context", {})
        file_scores = {}
        for entry in graph_context.get("symbol_context", []):
            for match in entry.get("matches", []):
                path = match.get("path", "")
                if not path:
                    continue
                file_scores[path] = file_scores.get(path, 0.0) + 0.35
        for path in graph_context.get("covering_tests", []):
            file_scores[path] = file_scores.get(path, 0.0) + 0.2
        candidate_files = [
            {"path": path, "reason": "graph_symbol_match", "score": min(score, 1.0)}
            for path, score in sorted(file_scores.items(), key=lambda item: item[1], reverse=True)
        ]
        entrypoints = [
            match["qualname"]
            for entry in graph_context.get("symbol_context", [])
            for match in entry.get("matches", [])
        ]
        return {
            "candidate_files": candidate_files[:6],
            "entrypoints": entrypoints[:6],
            "tests": graph_context.get("covering_tests", [])[:6],
            "confidence": 0.8 if candidate_files else 0.4,
        }
