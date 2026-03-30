FROM ubuntu:22.04

ARG DEBIAN_FRONTEND=noninteractive

# System deps for Tauri (GTK/webview build) + cross-compile to Windows (mingw-w64).
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    ca-certificates \
    git \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.0-dev \
    mingw-w64 \
    && rm -rf /var/lib/apt/lists/*

# Node.js (frontend build)
ARG NODE_MAJOR=20
RUN curl -fsSL https://deb.nodesource.com/setup_${NODE_MAJOR}.x | bash - \
    && apt-get update \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

# Rust toolchain
ENV CARGO_HOME=/root/.cargo
ENV RUSTUP_HOME=/root/.rustup
ENV PATH="${CARGO_HOME}/bin:${PATH}"

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --default-toolchain stable

# Add Windows GNU target for cross compilation.
RUN rustup target add x86_64-pc-windows-gnu

# Tauri CLI
RUN npm install -g @tauri-apps/cli

WORKDIR /work

CMD ["bash", "-lc", "echo Ready"]

