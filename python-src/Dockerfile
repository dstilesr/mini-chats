FROM ghcr.io/astral-sh/uv:python3.13-bookworm-slim

WORKDIR /app
ENV APP_STATIC_PATH=/app/assets
ENV APP_PORT=3501
EXPOSE 3501

COPY python-src/pyproject.toml .
COPY python-src/uv.lock .

RUN uv sync --no-cache --locked

COPY assets ./assets
COPY python-src/src .

CMD ["uv", "run", "main.py"]
