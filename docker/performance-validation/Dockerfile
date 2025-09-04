FROM ubuntu:22.04

# Install performance profiling tools
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    llvm-dev \
    libclang-dev \
    clang \
    cmake \
    valgrind \
    perf-tools-unstable \
    linux-tools-generic \
    hyperfine \
    time \
    && rm -rf /var/lib/apt/lists/*

# Create lambda user
RUN groupadd -r lambda && useradd -r -g lambda -m lambda
RUN mkdir -p /workspace && chown lambda:lambda /workspace

# Switch to lambda user
USER lambda
ENV PATH="/home/lambda/.cargo/bin:${PATH}"
ENV CARGO_TARGET_DIR="/workspace/target"

# Install Rust with performance tools
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN rustup default stable
RUN rustup component add clippy rustfmt
RUN cargo install cargo-flamegraph cargo-benchcmp

# Performance testing configuration
ENV RUST_BACKTRACE=1
ENV CARGO_INCREMENTAL=0

WORKDIR /workspace

# Create performance validation script
COPY --chown=lambda:lambda performance-test.sh /home/lambda/scripts/
RUN chmod +x /home/lambda/scripts/performance-test.sh