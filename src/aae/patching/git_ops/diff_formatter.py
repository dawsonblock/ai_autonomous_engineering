from __future__ import annotations

import difflib


class DiffFormatter:
    def create_diff(self, old_text: str, new_text: str, path: str) -> str:
        return "".join(
            difflib.unified_diff(
                old_text.splitlines(True),
                new_text.splitlines(True),
                fromfile="a/%s" % path,
                tofile="b/%s" % path,
            )
        )
