class PoiError(Exception):
    pass


class ApiError(PoiError):
    def __init__(self, status_code: int, message: str) -> None:
        self.status_code = status_code
        self.message = message
        super().__init__(f"HTTP {status_code}: {message}")


class NotFoundError(ApiError):
    def __init__(self, message: str = "Resource not found") -> None:
        super().__init__(404, message)


class AuthError(ApiError):
    def __init__(self, message: str = "Authentication failed") -> None:
        super().__init__(401, message)


class RateLimitError(ApiError):
    def __init__(self, message: str = "Rate limit exceeded") -> None:
        super().__init__(429, message)


class ValidationError(PoiError):
    pass


class TimeoutError(PoiError):
    pass
