from __future__ import annotations

from aae.contracts.micro_agents import PatchGenerationRequest
from aae.patching.patch_synthesizer import PatchSynthesizer


class HybridPatchGenerator:
    def __init__(self, llm_provider=None, synthesizer: PatchSynthesizer | None = None) -> None:
        self.synthesizer = synthesizer or PatchSynthesizer(provider=llm_provider)

    def generate(self, repo_path: str, request: PatchGenerationRequest) -> tuple[str, str]:
        original_text, updated_text, _ = self.synthesizer.synthesize(repo_path, request)
        return original_text, updated_text
