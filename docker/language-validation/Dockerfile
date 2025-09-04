# Language Processor Memory Safety Validation
FROM ubuntu:22.04

# Install essential tools for language processing validation
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
    gdb \
    strace \
    && rm -rf /var/lib/apt/lists/*

# Install Rust with memory sanitizer support
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install nightly toolchain for sanitizers
RUN rustup toolchain install nightly
RUN rustup component add rust-src --toolchain nightly
RUN rustup target add x86_64-unknown-linux-gnu --toolchain nightly

# Install Miri for unsafe code validation
RUN rustup component add miri --toolchain nightly

# Create non-root user for safer testing
RUN groupadd -r lambda && useradd -r -g lambda lambda
RUN mkdir -p /workspace && chown lambda:lambda /workspace

# Set up working directory
WORKDIR /workspace

# Language-specific environment variables
ENV RUST_BACKTRACE=full
ENV RUST_LOG=debug
ENV MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-symbolic-alignment-check"
ENV ASAN_OPTIONS="detect_stack_use_after_return=true:check_initialization_order=true:halt_on_error=true"

# Switch to lambda user for validation
USER lambda

# Final toolchain setup
RUN rustup default stable
RUN rustup component add clippy rustfmt

CMD ["/bin/bash"]