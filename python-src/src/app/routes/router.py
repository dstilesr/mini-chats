from fastapi import APIRouter, WebSocket

router = APIRouter(prefix="/api")


@router.websocket("/connect")
async def handle_connection(websocket: WebSocket):
    await websocket.accept()
    while True:
        msg = await websocket.receive()
