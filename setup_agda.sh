#!/bin/bash

# Agda Development Environment Setup Script
# For Lambdust Formal Verification

set -e

echo "🏥 Dr. Claude's Agda Surgery Environment Setup"
echo "=============================================="

echo "📦 Step 1: Installing GHC and Cabal..."
brew install ghc cabal-install

echo "📦 Step 2: Updating Cabal package list..."
cabal update

echo "📦 Step 3: Installing Agda..."
# Agdaをcabalでインストール
cabal install Agda

echo "📦 Step 4: Installing Agda Standard Library..."
# Agda標準ライブラリをダウンロード
cd /tmp
git clone https://github.com/agda/agda-stdlib.git
cd agda-stdlib

# 最新の安定版をチェックアウト
git checkout v1.7.3

# Agdaライブラリディレクトリを作成
mkdir -p ~/.agda
echo "/tmp/agda-stdlib/standard-library.agda-lib" >> ~/.agda/libraries
echo "standard-library" >> ~/.agda/defaults

echo "🔧 Step 5: Verifying Agda installation..."
agda --version

echo "🧪 Step 6: Testing Agda with simple proof..."
cat > /tmp/test.agda << 'EOF'
module test where

open import Data.Nat
open import Relation.Binary.PropositionalEquality

-- Simple proof that addition is commutative for 0
proof : ∀ (n : ℕ) → n + 0 ≡ 0 + n
proof zero = refl
proof (suc n) = cong suc (proof n)
EOF

agda --safe /tmp/test.agda

echo "✅ Agda installation completed successfully!"
echo ""
echo "🎯 Next steps:"
echo "1. Navigate to lambdust/agda directory"
echo "2. Start implementing R7RS formal semantics"
echo "3. Begin Phase II of the surgical plan"