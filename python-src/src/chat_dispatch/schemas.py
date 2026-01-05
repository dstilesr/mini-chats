from typing import Literal, NotRequired, TypedDict

from pydantic import BaseModel, TypeAdapter


class PublishedMessage(TypedDict):
    """
    Message to send to users when something is published to a channel.
    """

    sender: str
    channel_name: str
    sent_at: str
    content: str


class ServerResponse(TypedDict):
    """
    General response from server
    """

    status: Literal["ok", "error"]
    info: NotRequired[dict]


class SubscribeParams(BaseModel):
    """
    Parameters to subscribe to a channel
    """

    channel_name: str


class SubscribeRequest(BaseModel):
    """
    Request to subscribe to a channel.
    """

    action: Literal["subscribe"] = "subscribe"
    params: SubscribeParams


class SendParams(BaseModel):
    """
    Parameters to send a message to a channel
    """

    channel_name: str
    content: str


class SendRequest(BaseModel):
    """
    Request to send a message to a channel.
    """

    action: Literal["publish"] = "publish"
    params: SendParams


class UnSubscribeParams(BaseModel):
    """
    Parameters to unsubscribe from a channel
    """

    channel_name: str


class UnSubscribeRequest(BaseModel):
    """
    Request to unsubscribe from a channel.
    """

    action: Literal["unsubscribe"] = "unsubscribe"
    params: UnSubscribeParams


class ListRequest(BaseModel):
    """
    Request to list the channels a client is subscribed to.
    """

    action: Literal["list"] = "list"


RequestType = UnSubscribeRequest | SubscribeRequest | SendRequest | ListRequest
RequestAdapter = TypeAdapter(RequestType)
