FROM ubuntu:22.04

# 基本パッケージのインストール
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
    && rm -rf /var/lib/apt/lists/*

# 作業ディレクトリの設定
WORKDIR /workspace

# ユーザー権限での実行環境構築
RUN groupadd -r lambda && useradd -r -g lambda -m lambda
RUN mkdir -p /workspace && chown lambda:lambda /workspace

# lambdaユーザーでのRust環境構築
USER lambda
ENV PATH="/home/lambda/.cargo/bin:${PATH}"

# lambdaユーザー用のRustインストール
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN rustup default stable
RUN rustup component add clippy rustfmt

# 作業ディレクトリに戻る
WORKDIR /workspace