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

    def apply_llm_edits(self, file_path: str, original_text: str, llm_output: str) -> str:
        """Applies edits from LLM output. Attempts SEARCH/REPLACE, then UDIFF, then whole file."""
        import re

        # 1. Try SEARCH/REPLACE blocks
        sr_pattern = re.compile(
            r"<<<<<<<\s*SEARCH\n(.*?)\n=======\n(.*?)\n>>>>>>>\s*REPLACE", re.DOTALL
        )
        blocks = sr_pattern.findall(llm_output)
        if blocks:
            current_text = original_text
            for search, replace in blocks:
                search_exact = search + "\n" if not search.endswith("\n") else search
                replace_exact = replace + "\n" if not replace.endswith("\n") else replace
                if search_exact in current_text:
                    current_text = current_text.replace(search_exact, replace_exact, 1)
                else:
                    # try without final newline
                    if search in current_text:
                        current_text = current_text.replace(search, replace, 1)
            # If at least one block was matched and changed, we consider it a success.
            if current_text != original_text:
                 return current_text

        # 2. Try strict Unified Diff
        if "--- a/" in llm_output and "+++ b/" in llm_output and "@@" in llm_output:
            # We don't have a robust pure-python unified diff applier built-in, 
            # but we can try patch library or simple application.
            # Assuming we fall back to whole file if unified diff is too complex
            # For this simple prototype, let's use a very basic patch applicability if `patch` command is available, 
            # but for now we'll just return llm_output to trigger the next fallback.
            pass

        # 3. Fallback to Whole File Replacement
        # Check if the LLM output is wrapped in markdown code blocks
        code_block_pattern = re.compile(r"```[^\n]*\n(.*?)```", re.DOTALL)
        blocks = code_block_pattern.findall(llm_output)
        if blocks:
             # return the longest block as the likely file content
             return max(blocks, key=len)

        return llm_output
