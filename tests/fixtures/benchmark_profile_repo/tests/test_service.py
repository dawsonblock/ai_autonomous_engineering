from service import create_profile


def test_create_profile_valid_username():
    assert create_profile(" Demo ") == {"username": "demo"}


def test_create_profile_none_username_returns_none():
    assert create_profile(None) is None
