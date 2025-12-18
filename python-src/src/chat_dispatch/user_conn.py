import secrets
from asyncio import Queue
from dataclasses import dataclass, field
from typing import Self

from .schemas import PublishedMessage, RequestType, ServerResponse


@dataclass
class ClientConnection:
    """
    Wraps a connection to a client socket
    """

    client_name: str
    input_queue: Queue[tuple[str, RequestType]]
    publish_queue: Queue[PublishedMessage] = field(default_factory=Queue)
    rsp_queue: Queue[ServerResponse] = field(default_factory=Queue)

    @classmethod
    def create(
        cls,
        dispatch_queue: Queue[tuple[str, RequestType]],
        name: str | None = None,
    ) -> Self:
        """
        Instantiate a new client connection.
        """
        return cls(
            client_name=name or secrets.token_urlsafe(24),
            input_queue=dispatch_queue,
        )
