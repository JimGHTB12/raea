# The Linux build root (canary.2, canary.3): Ubuntu 24.04 x86_64 (glibc 2.39, as linux-check.sh's noble root), run under Rosetta
# (bun is its x64 baseline build: the default one segfaults under Rosetta)
FROM --platform=linux/amd64 ubuntu:24.04
RUN apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends \
      ca-certificates curl git build-essential pkg-config cmake ninja-build nasm clang libclang-dev lld zsh unzip jq rsync \
      binutils file procps python3 libasound2-dev libudev-dev libxkbcommon-dev libwayland-dev libx11-dev libxi-dev \
      libxcursor-dev libxrandr-dev libvulkan-dev libssl-dev libfontconfig-dev mesa-vulkan-drivers ffmpeg \
      xvfb xauth libxkbcommon-x11-0 \
    && rm -rf /var/lib/apt/lists/*
ARG BUN
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y -q --profile minimal --default-toolchain 1.99.1 \
    && curl -fsSLo /tmp/bun.zip https://github.com/oven-sh/bun/releases/download/bun-v$BUN/bun-linux-x64-baseline.zip \
    && mkdir -p /root/.bun/bin && unzip -qj /tmp/bun.zip -d /root/.bun/bin && rm /tmp/bun.zip
ENV PATH=/root/.cargo/bin:/root/.bun/bin:$PATH
