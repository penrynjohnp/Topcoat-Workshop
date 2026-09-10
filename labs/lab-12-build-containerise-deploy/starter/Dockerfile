# syntax=docker/dockerfile:1.7

FROM docker.io/library/rust:1.98.0-bookworm AS builder
WORKDIR /workspace
RUN cargo install topcoat-cli --version 0.7.0 --locked
COPY . .
# TODO(lab-12): Build the release binary and bundle assets from that same build.
RUN cargo build --release -p lab12-starter && \
    topcoat asset bundle --release -p lab12-starter && \
    cp target/release/lab12-starter /tmp/slipway && \
    cp -a target/release/assets /tmp/assets

FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app
# TODO(lab-12): Keep the binary and its matching assets directory together.
COPY --from=builder --chown=nonroot:nonroot /tmp/slipway ./slipway
COPY --from=builder --chown=nonroot:nonroot /tmp/assets ./assets
ENV HOST=0.0.0.0 PORT=8080 RUST_LOG=info
EXPOSE 8080
USER nonroot
ENTRYPOINT ["/app/slipway"]
