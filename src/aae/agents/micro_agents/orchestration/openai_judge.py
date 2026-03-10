from __future__ import annotations

import os
from typing import Any, Dict

import httpx

from aae.contracts.planner import CandidatePlan

class OpenAIJudgeProvider:
    def __init__(
        self,
        api_key: str,
        model: str = "gpt-4o-mini",
        base_url: str = "https://api.openai.com/v1",
        timeout_s: float = 30.0,
    ) -> None:
        self.api_key = api_key
        self.model = model
        self.base_url = base_url.rstrip("/")
        self.timeout_s = timeout_s

    @classmethod
    def from_env(cls) -> "OpenAIJudgeProvider | None":
        api_key = os.getenv("OPENAI_API_KEY", "").strip()
        if not api_key:
            return None
        return cls(
            api_key=api_key,
            model=os.getenv("AAE_JUDGE_MODEL", "gpt-4o-mini"),
            base_url=os.getenv("OPENAI_BASE_URL", "https://api.openai.com/v1"),
            timeout_s=float(os.getenv("AAE_JUDGE_TIMEOUT_S", "30")),
        )

    def select_best_plan(self, candidates: list[CandidatePlan]) -> str:
        prompt = self._build_prompt(candidates)
        payload: Dict[str, Any] = {
            "model": self.model,
            "messages": [
                {"role": "system", "content": "You are a senior software architect."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.0,
        }
        headers = {
            "Authorization": "Bearer %s" % self.api_key,
            "Content-Type": "application/json",
        }
        with httpx.Client(timeout=self.timeout_s) as client:
            response = client.post("%s/chat/completions" % self.base_url, headers=headers, json=payload)
            response.raise_for_status()
            data = response.json()
            
        output_text = ""
        try:
            output_text = data["choices"][0]["message"]["content"].strip()
        except (KeyError, IndexError, TypeError):
            raise ValueError("OpenAI judge provider returned unexpected response format: %s" % data)
            
        if not output_text:
            raise ValueError("OpenAI judge provider returned no output")
            
        # Extract the plan_id from the LLM output
        # The LLM is instructed to return ONLY the plan_id.
        # We'll check if any of the candidate plan_ids is present in the output.
        for candidate in candidates:
            if candidate.plan_id in output_text:
                return candidate.plan_id
                
        # Fallback to the first ID if extraction fails
        return candidates[0].plan_id if candidates else ""

    def _build_prompt(self, candidates: list[CandidatePlan]) -> str:
        prompt = "Review the following candidate patch strategies for a defect and select the BEST one based on safety, simplicity, and correctness.\n"
        prompt += "Return ONLY the exact plan_id of your chosen plan. Do not include any other text.\n\n"
        
        for candidate in candidates:
            prompt += f"--- Plan ID: {candidate.plan_id} ---\n"
            prompt += f"Summary: {candidate.summary}\n"
            prompt += f"Files changed: {len(candidate.changed_files)}\n"
            prompt += f"Risk Score: {candidate.risk_score:.3f}\n"
            prompt += f"Risk Reasons: {', '.join(candidate.risk_reasons)}\n\n"
            
        return prompt
