import logging

from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles

from app.routes import api_router
from app.settings import AppSettings

settings = AppSettings()

logging.basicConfig(level=getattr(logging, settings.log_level), force=True)

app = FastAPI(
    title="Python Mini Chat",
    description="Python Mini-Chat server.",
    version=settings.version,
)

app.include_router(api_router, prefix="/api")

app.mount("/", StaticFiles(directory=settings.static_path))

if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=settings.port,
        reload=settings.environment == "dev",
    )
