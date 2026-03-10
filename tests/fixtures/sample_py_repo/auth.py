from jwt_utils import parse_token


def authenticate(token: str):
    payload = parse_token(token)
    if not payload:
        return None
    return {"sub": payload["sub"]}
