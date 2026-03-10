from __future__ import annotations

import difflib


class DiffConstructor:
    def build(self, file_path: str, original_text: str, updated_text: str) -> str:
        return "".join(
            difflib.unified_diff(
                original_text.splitlines(True),
                updated_text.splitlines(True),
                fromfile="a/%s" % file_path,
                tofile="b/%s" % file_path,
            )
        )
