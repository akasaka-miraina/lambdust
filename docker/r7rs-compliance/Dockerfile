# R7RS Compliance Validation Docker Environment
# 
# This Docker image provides a controlled environment for validating that
# SIGSEGV fixes preserve exact R7RS Scheme semantics. Includes memory
# sanitizers and debugging tools for comprehensive validation.

FROM ubuntu:22.04

# Avoid interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    libc6-dev \
    gcc \
    g++ \
    gdb \
    valgrind \
    strace \
    ltrace \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Rust with specific toolchain for R7RS compliance testing
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain stable \
    --component clippy rustfmt
ENV PATH="/root/.cargo/bin:${PATH}"

# Install additional Rust components for memory safety validation
RUN rustup component add miri
RUN rustup toolchain install nightly --component miri

# Create lambda user for safer testing
RUN useradd -m -s /bin/bash lambda
RUN usermod -aG sudo lambda
RUN echo "lambda ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

# Set up working directory
WORKDIR /workspace
RUN chown lambda:lambda /workspace

# Copy validation scripts
COPY r7rs-validation-suite.sh /home/lambda/scripts/
COPY memory-safety-r7rs-test.sh /home/lambda/scripts/
RUN chmod +x /home/lambda/scripts/*.sh
RUN chown -R lambda:lambda /home/lambda/scripts/

# Configure environment for R7RS compliance testing
ENV RUST_BACKTRACE=full
ENV RUST_MIN_STACK=8388608
ENV CARGO_TERM_COLOR=always
ENV MIRI_ISOLATION_ERROR=warn

# Set memory safety debugging flags
ENV RUSTFLAGS="-Z sanitizer=address -Z sanitizer=memory"
ENV CARGO_TARGET_DIR=/tmp/cargo-target

USER lambda
CMD ["/bin/bash"]