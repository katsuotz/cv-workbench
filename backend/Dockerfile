ARG WORKER_RUNTIME_IMAGE=local-worker-runtime

FROM rust:1.93-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

RUN cargo build --release --locked --bins

FROM debian:bookworm-slim AS api

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/latex-renderer-backend /usr/local/bin/latex-renderer-backend

EXPOSE 18732

CMD ["latex-renderer-backend"]

FROM debian:bookworm-slim AS local-worker-runtime

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends -y \
        ca-certificates \
        fontconfig \
        fonts-font-awesome \
        fonts-lmodern \
        fonts-roboto \
        fonts-roboto-slab \
        texlive-fonts-extra \
        texlive-fonts-recommended \
        texlive-latex-extra \
        texlive-xetex \
    && rm -rf /var/lib/apt/lists/*

FROM ${WORKER_RUNTIME_IMAGE} AS worker

WORKDIR /app

COPY --from=builder /app/target/release/latex-renderer-worker /usr/local/bin/latex-renderer-worker

CMD ["latex-renderer-worker"]
