import asyncio
import logging
from collections import defaultdict
from dataclasses import dataclass, field
from datetime import UTC, datetime
from functools import partial

from fastapi import WebSocket

from . import schemas as sch
from .task_runner import runner
from .user_conn import ClientConnection

logger = logging.getLogger(__name__)


@dataclass
class Dispatcher:
    """
    Class to handle the chat server dispatch.
    """

    receive_queue: asyncio.Queue[tuple[str, sch.RequestType]] = field(
        default_factory=asyncio.Queue
    )
    clients: dict[str, ClientConnection] = field(default_factory=dict)
    client_to_channels: defaultdict[str, set[str]] = field(
        default_factory=partial(defaultdict, set)
    )
    channel_to_clients: defaultdict[str, set[str]] = field(
        default_factory=partial(defaultdict, set)
    )
    lock: asyncio.Lock = field(default_factory=asyncio.Lock)

    def add_client(self, client_name: str | None = None) -> sch.ServerResponse:
        """
        Register a new client connection.
        """
        new_client = ClientConnection.create(client_name)
        self.clients[new_client.client_name] = new_client
        return sch.ServerResponse(
            status="ok", info={"client_name": new_client.client_name}
        )

    async def subscribe(
        self, client_name: str, req: sch.SubscribeRequest
    ) -> sch.ServerResponse:
        """
        Subscribe a client to a channel.
        """
        async with self.lock:
            if client_name not in self.clients:
                logger.error(
                    "Client name '%s' not found among existing clients",
                    client_name,
                )
                return sch.ServerResponse(
                    status="error", info={"detail": "Client not found"}
                )

            channel = req.params.channel_name
            if not channel:
                logger.error("Invalid channel name (empty)")
                return sch.ServerResponse(
                    status="error",
                    info={"detail": "Empty channel name given"},
                )

            self.channel_to_clients[channel].add(client_name)
            self.client_to_channels[client_name].add(channel)
            return sch.ServerResponse(
                status="ok",
                info={
                    "channel_name": channel,
                    "total_subscribers": len(self.channel_to_clients[channel]),
                },
            )

    async def unsubscribe(
        self, client_name: str, req: sch.UnSubscribeRequest
    ) -> sch.ServerResponse:
        """
        Unsubscribe a client from a channel.
        """
        async with self.lock:
            if client_name not in self.clients:
                logger.error(
                    "Client name '%s' not found among existing clients",
                    client_name,
                )
                return sch.ServerResponse(
                    status="error", info={"detail": "Client not found"}
                )

            channel = req.params.channel_name
            if not channel:
                logger.error(
                    "Empty channel name given!",
                )
                return sch.ServerResponse(
                    status="error", info={"detail": "Empty channel name"}
                )

            self.channel_to_clients[channel].remove(client_name)
            self.client_to_channels[client_name].remove(channel)

            if len(self.channel_to_clients[channel]) == 0:
                self.channel_to_clients.pop(channel)

        return sch.ServerResponse(status="ok")

    async def publish_msg(
        self, client_name: str, req: sch.SendRequest
    ) -> sch.ServerResponse:
        """
        Publish a message from a client to a channel.
        """
        msg = sch.PublishedMessage(
            sender=client_name,
            sent_at=datetime.now(UTC).isoformat(),
            channel_name=req.params.channel_name,
            content=req.params.content,
        )
        if client_name not in self.clients:
            logger.error("Unknown client name: %s", client_name)
            return sch.ServerResponse(
                status="error", info={"detail": "Unknown client name"}
            )

        async with self.lock:
            channel = req.params.channel_name
            for subscriber in self.channel_to_clients[channel]:
                if subscriber == client_name:
                    continue

                client = self.clients[subscriber]
                runner.dispatch_task(client.publish_queue.put(msg))
                logger.debug(
                    "Message dispatched to subscriber '%s'", subscriber
                )

        return sch.ServerResponse(status="ok")

    async def list_subscribed_channels(
        self, client_name: str
    ) -> sch.ServerResponse:
        """
        List the channels the user is subscribed to.
        """
        async with self.lock:
            if client_name not in self.clients:
                return sch.ServerResponse(
                    status="error", info={"detail": "Unknown client name"}
                )

            channels = list(self.client_to_channels.get(client_name, set()))
            channels.sort()
            return sch.ServerResponse(status="ok", info={"channels": channels})

    async def client_listener(self, client_name: str, sock: WebSocket):
        """
        Listener process for sending published messages to clients.
        """
        client = self.clients[client_name]
        while True:
            msg = await client.publish_queue.get()
            logger.debug(
                "Received message to publish to client '%s'", client_name
            )
            await sock.send_json(msg)

    async def remove_client(self, client_name: str):
        """
        Remove a client from the service
        """
        async with self.lock:
            self.clients.pop(client_name, None)
            channels = self.client_to_channels.pop(client_name, set())
            for chan in channels:
                self.channel_to_clients[chan].remove(client_name)
                if len(self.channel_to_clients[chan]) == 0:
                    self.channel_to_clients.pop(chan, None)
