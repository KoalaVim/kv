# ── Stage 1: build kv ────────────────────────────────────────────────
FROM rust:1-bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --release && strip target/release/kv

# ── Stage 2: runtime ────────────────────────────────────────────────
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# System packages (git, curl required by kv; tar/unzip for kv install;
# ripgrep/fd/fzf so health checks pass out of the box)
RUN apt-get update && apt-get install -y --no-install-recommends \
        git curl ca-certificates \
        tar unzip \
        ripgrep fd-find fzf \
    && ln -sf /usr/bin/fdfind /usr/bin/fd \
    && rm -rf /var/lib/apt/lists/*

# Neovim stable — pick the right binary for the build platform
ARG NVIM_VERSION=stable
RUN set -e; \
    ARCH="$(uname -m)"; \
    case "$ARCH" in \
        x86_64)  NVIM_ARCH=x86_64 ;; \
        aarch64) NVIM_ARCH=arm64  ;; \
        *)       echo "unsupported arch: $ARCH" >&2; exit 1 ;; \
    esac; \
    curl -fsSL \
        "https://github.com/neovim/neovim/releases/download/${NVIM_VERSION}/nvim-linux-${NVIM_ARCH}.tar.gz" \
        | tar xzf - -C /opt \
    && ln -sf /opt/nvim-linux-*/bin/nvim /usr/local/bin/nvim

# Copy kv from builder
COPY --from=builder /build/target/release/kv /usr/local/bin/kv

# Entrypoint script
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Non-root user
RUN useradd -m -s /bin/bash koala
USER koala
WORKDIR /home/koala

# Create the default "main" env from the KoalaVim config template
RUN kv env create main --from https://github.com/KoalaVim/KoalaConfig.template

# Pre-install lazy.nvim plugins so the image is ready to use.
# Failures here are non-fatal — the entrypoint retries on first run.
RUN NVIM_APPNAME=kvim-envs/main nvim --headless "+Lazy! sync" +qa 2>/dev/null; exit 0

ENTRYPOINT ["entrypoint.sh"]
