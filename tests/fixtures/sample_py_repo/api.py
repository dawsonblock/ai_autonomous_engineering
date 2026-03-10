from auth import authenticate


def login(token: str):
    return authenticate(token)
