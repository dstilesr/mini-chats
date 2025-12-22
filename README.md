# Mini Chats
This repository contains implementations of mini chat (or rather pub-sub) servers with the same 
functionality in several programming languages for practice using the languages and as a reference
for other projects. The servers will all include the same basic functionality:

- They will communicate with clients via a web socket connection.
- They will expose the following functionality to the client:
  1. A client can subscribe to a channel. If the channel does not already exist, it will be
    created. If a channel has no active subscribers, it will be deleted.
  2. A client can list existing channels.
  3. A client can publish a message to a channel they are subscribed to.
  4. A client will receive any messages sent to channels they are subscribed to.
  5. A client can unsubscribe from a channel.
  
## Message Specifications

The following API contracts will be used by all implementations. The endpoint to connect to the
server via a socket will be `/api/connect`.

### Establish Connection
When establishing a connection from a client, the client may send a `client_name` query parameter
to be associated to their messages (e.g., `/api/connect?client_name=Alice`). If this parameter is
not given, a random name of some sort will be assigned to them. When a connection is established, the server will send
a confirmation with this schema:
```json
{
  "status": "ok",
  "info": {
    "client_name": "<given client name>"
  }
}
```

### Subscribe to Channel
To subscribe to a channel, the client must send a request with this schema:
```json
{
  "action": "subscribe",
  "params": {
    "channel_name": "<channel name>"
  }
}
```

The server will send a confirmation with this schema:
```json
{
  "status": "ok",
  "info": {
    "channel_name": "<channel name>",
    "total_subscribers": 1
  }
}
```

### Send Message

#### Dispatch
To publish a message to a channel, the client must send a message over the socket with this schema:
```json
{
  "action": "publish",
  "params": {
    "channel_name": "<channel name>",
    "content": "<Message Content>"
  }
}
```

The server will in turn respond with
```json
{
  "status": "ok"
}
```

#### Reception
When a message is dispatched by a client, all subscribers to the channel will get the following message from the server:
```json
{
  "sender": "<given client name>",
  "channel_name": "<channel name>",
  "sent_at": "<timestamp when message was received by server - ISO format with timezone>",
  "content": "<Message Content>"
}
```

### Unsubscribe from Channel
To unsubscribe from a channel, the client must send a request with this schema:
```json
{
  "action": "unsubscribe",
  "params": {
    "channel_name": "<channel name>"
  }
}
```

The server will send a confirmation with this schema:
```json
{
  "status": "ok"
}
```

### Error Messages
When a message sent by a client is invalid or results in an error on the server, the server will
send a response in this format:
```json
{
  "status": "error",
  "info": {
    "detail": "A description / explanation of the error"
  }
}
```

## Configurations
Basic confirgurations for the server will be given by the following environment variables:
- `APP_PORT` - Default: 3501
- `APP_VERSION`
- `APP_ENVIRONMENT` - Default: `dev`

## Implementations

- [`Python`](./python-src/README.md)
