FROM debian:bookworm-slim

WORKDIR /app

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
