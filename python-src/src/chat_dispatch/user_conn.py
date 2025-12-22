import secrets
from asyncio import Queue
from dataclasses import dataclass, field
from typing import Self

from .schemas import PublishedMessage


@dataclass
class ClientConnection:
    """
    Wraps a connection to a client socket
    """

    client_name: str
    publish_queue: Queue[PublishedMessage] = field(default_factory=Queue)

    @classmethod
    def create(
        cls,
        name: str | None = None,
    ) -> Self:
        """
        Instantiate a new client connection.
        """
        return cls(
            client_name=name or secrets.token_urlsafe(24),
        )
