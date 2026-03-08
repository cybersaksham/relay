FROM rust:1.88-bookworm AS builder
WORKDIR /app

COPY Cargo.toml .
COPY migrations ./migrations
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates git openssh-client \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/relay-backend /usr/local/bin/relay-backend

CMD ["relay-backend"]
