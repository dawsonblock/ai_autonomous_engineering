from __future__ import annotations

import difflib


class DiffConstructor:
    def build(self, file_path: str, original_text: str, updated_text: str) -> str:
        diff = difflib.unified_diff(
            original_text.splitlines(),
            updated_text.splitlines(),
            fromfile="a/%s" % file_path,
            tofile="b/%s" % file_path,
            lineterm="",
        )
        return "\n".join(diff)
