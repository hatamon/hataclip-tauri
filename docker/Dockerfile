FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:/usr/local/bin:$PATH

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        build-essential \
        pkg-config \
        file \
        patchelf \
        libwebkit2gtk-4.1-dev \
        libgtk-3-dev \
        librsvg2-dev \
        libssl-dev \
        libayatana-appindicator3-dev \
        libx11-dev \
        libxdo-dev \
        libxi-dev \
        libxtst-dev \
        gcc-mingw-w64-x86-64 \
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal \
    && rustup target add x86_64-pc-windows-gnu \
    && rustup component add rustfmt

WORKDIR /app
