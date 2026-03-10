from aae.localization.failure_mapper import FailureMapper


def test_parse_pytest_output_basic():
    output = """
=================================== FAILURES ===================================
___________________________ test_auth_invalid_token ____________________________
FAILED src/auth.py:42: AuthenticationError
    def authenticate(token):
>       raise AuthenticationError("Invalid token")
E       AuthenticationError: Invalid token

src/auth.py:42: AuthenticationError
=========================== short test summary info ============================
FAILED tests/test_auth.py::test_auth_invalid_token - AuthenticationError: Invalid token
"""
    mapper = FailureMapper()
    failures = mapper.parse_pytest_output(output)

    assert len(failures) > 0

    f = failures[0]
    assert "test_auth_invalid_token" in f.test_name
    assert f.file_path == "src/auth.py"
    assert f.line_number == 42
    assert f.exception_type == "AuthenticationError"
