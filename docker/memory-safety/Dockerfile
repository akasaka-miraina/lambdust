FROM ubuntu:22.04

# Install basic packages and memory safety tools
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
    ltrace \
    && rm -rf /var/lib/apt/lists/*

# Create lambda user with proper permissions
RUN groupadd -r lambda && useradd -r -g lambda -m lambda
RUN mkdir -p /workspace && chown lambda:lambda /workspace

# Switch to lambda user for Rust installation
USER lambda
ENV PATH="/home/lambda/.cargo/bin:${PATH}"
ENV RUST_BACKTRACE=full
ENV MIRIFLAGS="-Zmiri-disable-isolation"

# Install Rust with memory safety components
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN rustup default stable
RUN rustup component add clippy rustfmt miri
RUN rustup toolchain install nightly --component miri

# Install AddressSanitizer and other memory safety tools
RUN rustup toolchain install nightly
RUN rustup component add rust-src --toolchain nightly

# Set working directory
WORKDIR /workspace

# Create memory safety testing script
RUN mkdir -p /home/lambda/scripts
COPY --chown=lambda:lambda memory-safety-test.sh /home/lambda/scripts/
RUN chmod +x /home/lambda/scripts/memory-safety-test.sh