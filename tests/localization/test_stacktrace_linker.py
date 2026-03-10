from aae.localization.stacktrace_linker import StacktraceLinker


def test_stacktrace_linker_parse():
    trace = '''
Traceback (most recent call last):
  File "/absolute/path/to/repo/src/main.py", line 10, in execute
    result = login()
  File "/absolute/path/to/repo/src/auth.py", line 25, in login
    raise ValueError("Failed")
'''
    linker = StacktraceLinker()
    frames = linker.parse(trace, "/absolute/path/to/repo")

    assert len(frames) == 2
    assert frames[0].file_path == "src/main.py"
    assert frames[0].function_name == "execute"
    assert frames[0].line_number == 10

    assert frames[1].file_path == "src/auth.py"
    assert frames[1].function_name == "login"
    assert frames[1].line_number == 25
