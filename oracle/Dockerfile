# Build stage
FROM rust:1.83-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

# Runtime stage — imagen mínima, red interna
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/oracle-authorization /usr/local/bin/oracle-authorization

ENV ORACLE_HOST=0.0.0.0
ENV ORACLE_PORT=8081

EXPOSE 8081

# Solo red interna; no publicar este puerto a Internet en producción
CMD ["oracle-authorization"]
