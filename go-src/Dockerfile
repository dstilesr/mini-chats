FROM golang:1.24-alpine AS builder

WORKDIR /build

COPY go-src .

RUN go build -o minichat

FROM alpine:latest

ENV APP_PORT=3501
ENV APP_STATIC_PATH=/app/assets
EXPOSE 3501

WORKDIR /app
COPY assets ./assets
COPY --from=builder /build/minichat .

CMD ["./minichat"]
