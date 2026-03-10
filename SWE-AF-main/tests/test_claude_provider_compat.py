import unittest
from unittest.mock import AsyncMock

from claude_agent_sdk import ClaudeAgentOptions
from swe_af.agent_ai.providers.claude.client import (
    ClaudeProviderClient,
    _build_sdk_protocol_error_message,
    _is_sdk_protocol_error,
    _is_transient,
)


class ClaudeProviderCompatTests(unittest.TestCase):
    def test_rate_limit_event_is_protocol_error(self) -> None:
        err = "Unknown message type: rate_limit_event"
        self.assertTrue(_is_sdk_protocol_error(err))

    def test_rate_limit_event_is_not_treated_as_transient(self) -> None:
        err = "Unknown message type: rate_limit_event"
        self.assertFalse(_is_transient(err))

    def test_protocol_error_message_contains_sdk_guidance(self) -> None:
        err = "Unknown message type: rate_limit_event"
        msg = _build_sdk_protocol_error_message(err, sdk_version="0.1.39")
        self.assertIn("version=0.1.39", msg)
        self.assertIn("claude-agent-sdk==0.1.20", msg)

    def test_protocol_error_fails_fast_without_retries(self) -> None:
        err = "Unknown message type: rate_limit_event"
        client = ClaudeProviderClient()
        client._execute = AsyncMock(side_effect=RuntimeError(err))
        options = ClaudeAgentOptions(model="sonnet", cwd=".", max_turns=1)

        async def _run() -> None:
            await client._run_with_retries(
                prompt="test",
                final_prompt="test",
                options=options,
                output_schema=None,
                output_path=None,
                effective_cwd=".",
                effective_model="sonnet",
                effective_env={},
                effective_perm=None,
                effective_retries=3,
                temp_files=[],
                stderr_lines=[],
            )

        with self.assertRaises(RuntimeError) as ctx:
            import asyncio

            asyncio.run(_run())

        self.assertIn("claude-agent-sdk==0.1.20", str(ctx.exception))
        self.assertEqual(client._execute.await_count, 1)


if __name__ == "__main__":
    unittest.main()
