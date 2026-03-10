def parse_token(token: str):
    if token is None:
        return {}
    token = token.strip()
    if not token:
        raise ValueError("empty token")
    return {"sub": token}
