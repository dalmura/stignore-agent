FROM rust:1-slim-trixie AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

WORKDIR /app

COPY --from=builder /app/target/release/stignore-agent /stignore-agent

ENTRYPOINT ["/stignore-agent"]
CMD ["/app/config.toml"]
