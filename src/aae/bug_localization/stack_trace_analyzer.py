from __future__ import annotations

import re
from typing import List

from aae.contracts.localization import FailureEvidence


TRACE_RE = re.compile(r'File "?(?P<file>[^",]+)"?, line (?P<line>\d+), in (?P<symbol>[A-Za-z_][A-Za-z0-9_]*)')


class StackTraceAnalyzer:
    def parse(self, text: str) -> List[FailureEvidence]:
        evidence = []
        for match in TRACE_RE.finditer(text or ""):
            evidence.append(
                FailureEvidence(
                    source="stack_trace",
                    file_path=match.group("file"),
                    symbol=match.group("symbol"),
                    line=int(match.group("line")),
                    weight=0.9,
                    metadata={"raw": match.group(0)},
                )
            )
        return evidence
