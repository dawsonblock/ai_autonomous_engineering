from auth import authenticate


def test_authenticate_valid_token():
    assert authenticate(" demo ") == {"sub": "demo"}
