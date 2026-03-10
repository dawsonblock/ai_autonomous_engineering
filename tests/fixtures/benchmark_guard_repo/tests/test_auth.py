from auth import authenticate


def test_authenticate_valid_token():
    assert authenticate(" demo ") == {"sub": "demo"}


def test_authenticate_empty_token_returns_none():
    assert authenticate("") is None
