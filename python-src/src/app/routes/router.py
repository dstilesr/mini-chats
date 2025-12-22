import logging

from fastapi import APIRouter, WebSocket
from pydantic import ValidationError

from chat_dispatch import dispatcher
from chat_dispatch import schemas as sch
from chat_dispatch.task_runner import runner

logger = logging.getLogger(__name__)
dispatch_handler = dispatcher.Dispatcher()

router = APIRouter()


@router.websocket("/connect")
async def handle_connection(
    websocket: WebSocket,
    client_name: str | None = None,
):
    await websocket.accept()
    rsp = dispatch_handler.add_client(client_name)
    await websocket.send_json(rsp)

    if rsp["status"] == "ok":
        client_id = rsp["info"]["client_name"]  # type: ignore
        logger.info("Client %s connected", client_name)
    else:
        logger.error("Could not add client: %s", rsp["info"]["detail"])  # type: ignore
        await websocket.close()
        return

    # Start listener
    listen_task = runner.dispatch_task(
        dispatch_handler.client_listener(client_id, websocket)
    )

    async for msg in websocket.iter_text():
        try:
            logger.debug("Received message: %s", msg)
            parsed = sch.RequestAdapter.validate_json(msg)

            match type(parsed):
                case sch.SendRequest:
                    rsp = await dispatch_handler.publish_msg(client_id, parsed)
                case sch.SubscribeRequest:
                    rsp = await dispatch_handler.subscribe(client_id, parsed)
                case sch.UnSubscribeRequest:
                    rsp = await dispatch_handler.unsubscribe(client_id, parsed)
                case _:
                    logger.error(
                        "Unknown type of request: [%s.%s]",
                        type(parsed).__module__,
                        type(parsed).__name__,
                    )
                    continue

            await websocket.send_json(rsp)

        except ValidationError as e:
            logger.error("Could not parse request: %s", str(e))
            await websocket.send_json(
                sch.ServerResponse(status="error", info={"detail": str(e)})
            )

        except Exception as e:
            logger.error(
                "Error when processing message: [%s.%s] %s",
                type(e).__module__,
                type(e).__name__,
                str(e),
            )

    runner.stop_task(listen_task)
