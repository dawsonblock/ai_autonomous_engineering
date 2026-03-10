def parse_token(token: str):
    if not token:
        return {}
    return {"sub": token.strip()}
