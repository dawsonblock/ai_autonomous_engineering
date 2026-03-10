from formatter import normalize_username


def create_profile(username: str):
    value = normalize_username(username)
    if not value:
        return None
    return {"username": value}
