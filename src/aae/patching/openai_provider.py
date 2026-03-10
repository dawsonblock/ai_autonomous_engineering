from __future__ import annotations

import os
from typing import Any, Dict

import httpx

from aae.contracts.micro_agents import PatchGenerationRequest


class OpenAIPatchProvider:
    def __init__(
        self,
        api_key: str,
        model: str = "gpt-4.1-mini",
        base_url: str = "https://api.openai.com/v1",
        timeout_s: float = 30.0,
    ) -> None:
        self.api_key = api_key
        self.model = model
        self.base_url = base_url.rstrip("/")
        self.timeout_s = timeout_s

    @classmethod
    def from_env(cls) -> "OpenAIPatchProvider | None":
        api_key = os.getenv("OPENAI_API_KEY", "").strip()
        if not api_key:
            return None
        return cls(
            api_key=api_key,
            model=os.getenv("AAE_PATCH_MODEL", "gpt-4.1-mini"),
            base_url=os.getenv("OPENAI_BASE_URL", "https://api.openai.com/v1"),
            timeout_s=float(os.getenv("AAE_PATCH_TIMEOUT_S", "30")),
        )

    def generate_body(
        self,
        request: PatchGenerationRequest,
        function_signature: str,
        original_body: str,
        prompt_hint: str,
    ) -> str:
        prompt = self._prompt(request, function_signature, original_body, prompt_hint)
        payload: Dict[str, Any] = {
            "model": self.model,
            "input": prompt,
        }
        headers = {
            "Authorization": "Bearer %s" % self.api_key,
            "Content-Type": "application/json",
        }
        with httpx.Client(timeout=self.timeout_s) as client:
            response = client.post("%s/responses" % self.base_url, headers=headers, json=payload)
            response.raise_for_status()
            data = response.json()
        if isinstance(data.get("output_text"), str):
            return data["output_text"].strip()
        for item in data.get("output", []):
            for content in item.get("content", []):
                if content.get("type") == "output_text":
                    return str(content.get("text", "")).strip()
        raise ValueError("OpenAI patch provider returned no output_text")

    def _prompt(
        self,
        request: PatchGenerationRequest,
        function_signature: str,
        original_body: str,
        prompt_hint: str,
    ) -> str:
        return (
            "You are generating a bounded code edit.\n"
            "Return only the replacement body for the target Python function. "
            "Do not include the function definition, markdown, or explanations.\n\n"
            "File: {file_path}\n"
            "Symbol: {symbol}\n"
            "Signature: {signature}\n"
            "Strategy: {strategy}\n"
            "Template family: {template_family}\n"
            "Expected behavior: {expected_behavior}\n"
            "Prompt hint: {prompt_hint}\n"
            "Constraints: {constraints}\n"
            "Repair constraints: {repair_constraints}\n"
            "Declared allowed intents: {declared_allowed_intents}\n"
            "Suspicious context: {suspicious_context}\n\n"
            "Original body:\n{body}\n"
        ).format(
            file_path=request.file_path,
            symbol=request.symbol,
            signature=function_signature,
            strategy=request.strategy,
            template_family=request.template_family,
            expected_behavior=request.expected_behavior,
            prompt_hint=prompt_hint,
            constraints=request.constraints,
            repair_constraints=request.repair_constraints,
            declared_allowed_intents=request.declared_allowed_intents,
            suspicious_context=request.suspicious_context,
            body=original_body,
        )
