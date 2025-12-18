from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class AppSettings(BaseSettings):
    """
    Settings for the application.
    """

    environment: str = Field(default="dev")
    port: int = Field(default=3501, gt=0)
    version: str = Field(default="v0.1.0")

    model_config = SettingsConfigDict(env_prefix="APP_")
