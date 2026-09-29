# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.98.1
ARG ALPINE_VERSION=3.24
ARG APP_NAME=double_ues_rs

################################################################################
FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app
COPY . .

#RUN apk add --no-cache clang lld musl-dev openssl-libs-static git perl make curl

RUN --mount=type=cache,target=/app/target/,id=${APP_NAME} \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
cargo build --locked --release -p ${APP_NAME} && \
cp ./target/release/$APP_NAME /bin/$APP_NAME
################################################################################
FROM alpine:${ALPINE_VERSION} AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

COPY --from=build /bin/double_ues_rs /bin/

RUN mkdir -p /app/logs

USER appuser
EXPOSE 5252

CMD ["/bin/double_ues_rs"]

