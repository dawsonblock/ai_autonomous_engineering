from __future__ import annotations

from pathlib import Path


class TestMutator:
    def write_ephemeral_tests(self, artifact_dir: str, target_symbol: str, counterexamples: list[dict]) -> list[str]:
        root = Path(artifact_dir)
        root.mkdir(parents=True, exist_ok=True)
        if not counterexamples:
            return []
        path = root / "test_generated_counterexamples.py"
        lines = [
            "import pytest",
            "from %s import %s" % (target_symbol.split(".")[0], target_symbol.split(".")[-1] if "." in target_symbol else target_symbol),
            "",
        ]
        function_name = target_symbol.split(".")[-1]
        for case in counterexamples:
            test_name = "test_%s_%s" % (function_name, case["name"])
            value_literal = repr(case["input"])
            lines.extend(
                [
                    "def %s():" % test_name,
                    "    try:",
                    "        result = %s(%s)" % (function_name, value_literal),
                    "    except Exception:",
                    "        result = None",
                    "    assert result is None or result == {}",
                    "",
                ]
            )
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        return [str(path)]
