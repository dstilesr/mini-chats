from pathlib import Path
from typing import Literal

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class AppSettings(BaseSettings):
    """
    Settings for the application.
    """

    environment: str = Field(default="dev")
    port: int = Field(default=3501, gt=0)
    version: str = Field(default="v0.1.0")
    static_path: Path = Field(
        default_factory=lambda: Path(__file__).parents[3] / "assets",
        description="Path from which to serve static files",
    )
    log_level: Literal["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"] = Field(
        default="INFO"
    )

    model_config = SettingsConfigDict(env_prefix="APP_")
