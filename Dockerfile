# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.78
ARG APP_NAME=quote_server

FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app

# Install host build dependencies.
ENV DATABASE_URL=sqlite:db/quotes.db

RUN apk add --no-cache clang lld musl-dev git curl

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=build.rs,target=build.rs \
    --mount=type=bind,source=askama.toml,target=askama.toml \
    --mount=type=bind,source=assets,target=assets \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=bind,source=.sqlx,target=.sqlx \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release && \
    cp ./target/release/$APP_NAME /bin/server

FROM alpine:latest AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=build /bin/server /bin/
COPY --chown=appuser:appuser ./assets ./assets

EXPOSE 3000

# What the container should run when it is started.
CMD ["/bin/server", "--init-from", "assets/static/quotes.json"]
