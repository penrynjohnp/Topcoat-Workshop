# syntax=docker/dockerfile:1.7

# ANCHOR: release-container
FROM docker.io/library/rust:1.98.0-bookworm AS builder
WORKDIR /workspace
RUN cargo install topcoat-cli --version 0.7.0 --locked
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/workspace/target \
    cargo build --release -p slipway && \
    topcoat asset bundle --release -p slipway && \
    cp target/release/slipway /tmp/slipway && \
    cp -a target/release/assets /tmp/assets

FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app
COPY --from=builder --chown=nonroot:nonroot /tmp/slipway ./slipway
COPY --from=builder --chown=nonroot:nonroot /tmp/assets ./assets
ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=info \
    SLIPWAY_ENV=production
EXPOSE 8080
USER nonroot
ENTRYPOINT ["/app/slipway"]
# ANCHOR_END: release-container
