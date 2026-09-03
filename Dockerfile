# build
FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# cache deps separately from source - build a throwaway main.rs
# against just the manifest first, so `cargo build` layer only
# reruns when Cargo.toml/Cargo.lock actually change.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --bin mantle
RUN rm -rf src

COPY src ./src
# touch so cargo doesn't skip the real build using the dummy's mtime
RUN touch src/main.rs
RUN cargo build --release --bin mantle

# minimal runtime image
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system mantle \
    && useradd --system --gid mantle --no-create-home --shell /usr/sbin/nologin mantle

WORKDIR /app
COPY --from=builder /app/target/release/mantle /usr/local/bin/mantle
COPY migrations ./migrations

USER mantle

ENTRYPOINT ["/usr/local/bin/mantle"]
