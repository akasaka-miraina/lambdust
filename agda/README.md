# Lambdust Agda形式的検証

## 概要

LambdustのすべてのOptimizerはAgdaによる形式的証明を必要とします。
ここではR7RS Scheme形式的意味論とOptimizer証明を記述します。

## ディレクトリ構造

```
agda/
├── R7RS/                 # R7RS形式的意味論
│   ├── Core.agda        # 基本データ型・環境・評価
│   ├── Continuations.agda # CPS意味論
│   ├── Primitives.agda  # 組み込み関数
│   └── All.agda         # 全モジュールのインポート
├── Optimizations/        # 最適化の形式的証明
│   ├── ConstantFolding.agda
│   ├── InlineContinuation.agda
│   ├── TailCall.agda
│   └── All.agda
├── Properties/           # 一般的性質の証明
│   ├── Equivalence.agda # 意味論的等価性
│   ├── Determinism.agda # 決定性
│   └── All.agda
└── Utils/               # 共通ユーティリティ
    ├── List.agda
    ├── String.agda
    └── All.agda
```

## 開発ルール

1. **新しい最適化の追加**
   - まずAgdaで形式的証明を作成
   - `agda --safe`で証明を検証
   - 証明完了後のみRust実装

2. **証明の要件**
   - R7RS意味論との等価性証明必須
   - 停止性証明（可能な場合）
   - 型安全性証明

3. **実装同期**
   - Agda証明の変更時はRust実装も更新
   - CI/CDで自動同期確認

## セットアップ

```bash
# Agdaインストール
cabal update
cabal install Agda

# 証明の確認
cd agda
agda --safe R7RS/All.agda
agda --safe Optimizations/All.agda
```

## 現在の状況

- [ ] R7RS基本意味論モデル化
- [ ] Expression Analyzer最適化の証明
- [ ] 継続最適化の証明
- [ ] Rust実装との同期システム