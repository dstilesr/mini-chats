from typing import Type, TypeVar

T = TypeVar("T")


class Singleton(type):
    """
    Metaclass for creating singletons.
    """

    _instances: dict[Type[T], T] = {}  # type: ignore

    def __new__(cls, *args, **kwargs):
        if cls not in cls._instances:
            cls._instances[cls] = super().__new__(cls, *args, **kwargs)
        return cls._instances[cls]  # type: ignore
