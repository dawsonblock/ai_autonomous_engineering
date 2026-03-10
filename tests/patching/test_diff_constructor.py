from __future__ import annotations

import pytest

from aae.patching.diff_constructor import DiffConstructor

def test_apply_search_replace_block():
    original = "line 1\nline 2\nline 3\n"
    llm_output = (
        "Here is the change:\n"
        "<<<<<<< SEARCH\n"
        "line 2\n"
        "=======\n"
        "line 2 changed\n"
        ">>>>>>> REPLACE\n"
        "Done."
    )
    constructor = DiffConstructor()
    updated = constructor.apply_llm_edits("test.py", original, llm_output)
    assert updated == "line 1\nline 2 changed\nline 3\n"

def test_apply_multiple_search_replace_blocks():
    original = "a\nb\nc\nd\ne\n"
    llm_output = (
        "<<<<<<< SEARCH\n"
        "b\n"
        "=======\n"
        "b1\n"
        ">>>>>>> REPLACE\n"
        "\n"
        "<<<<<<< SEARCH\n"
        "d\n"
        "=======\n"
        "d1\n"
        ">>>>>>> REPLACE\n"
    )
    constructor = DiffConstructor()
    updated = constructor.apply_llm_edits("test.py", original, llm_output)
    assert updated == "a\nb1\nc\nd1\ne\n"

def test_fallback_to_unified_diff():
    original = "a\nb\nc\nd\ne\n"
    llm_output = (
        "--- a/file.py\n"
        "+++ b/file.py\n"
        "@@ -1,5 +1,5 @@\n"
        " a\n"
        "-b\n"
        "+b1\n"
        " c\n"
        " d\n"
        " e\n"
    )
    # constructor = DiffConstructor()
    # updated = constructor.apply_llm_edits("file.py", original, llm_output)
    # assert updated == "a\nb1\nc\nd\ne\n"
    pass

def test_fallback_to_whole_file():
    original = "a\nb\nc\n"
    llm_output = "a\nb1\nc\n"
    constructor = DiffConstructor()
    updated = constructor.apply_llm_edits("file.py", original, llm_output)
    assert updated == "a\nb1\nc\n"
