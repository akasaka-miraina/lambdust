<!-- for GitHub Copilot review rule -->

<details>
<summary>for GitHub Copilot review rule</summary>

- Please provide suggestions for improving code quality in terms of
  - Logic accuracy
  - Security issues
  - Performance optimization
  - Potential data conflicts
  - Maintain consistency
  - Error handling
  - Improved maintainability
  - Ensure modularity
  - Reduce code complexity
  - Optimization possibilities
  - Apply best practices (DRY, SOLID, KISS)
- Focus your review on critical issues (e.g., test failures, vulnerabilities, bottlenecks above O(n²))
- Do not comment on
  - Minor code style issues
  - Lack of comments/documentation
</details>
<!-- for GitHub Copilot review rule -->

## 概要

<!-- このPRで実装した内容を簡潔に説明してください -->

## 関連Issue

<!-- 対応するIssueを記載してください -->
Closes #

## 変更内容

<!-- 変更した内容を詳細に記述してください -->

### 追加機能

- [ ] 機能1: 説明
- [ ] 機能2: 説明

### 修正内容

- [ ] 修正1: 説明
- [ ] 修正2: 説明

### 変更されたファイル

<!-- 主要な変更ファイルとその変更理由 -->

- `src/file1.rs`: 変更理由
- `src/file2.rs`: 変更理由
- `tests/test_file.rs`: 新規テストの追加

## テスト

### 追加されたテスト

<!-- 新しく追加したテストについて説明してください -->

- [ ] 単体テスト: `test_function_name`
- [ ] 統合テスト: `test_integration_scenario`
- [ ] パフォーマンステスト: 実行時間計測

### テスト結果

```bash
# ローカルでのテスト実行結果
cargo test
# 結果: XX passed; 0 failed
```

### テストカバレッジ

<!-- 新機能のテストカバレッジについて -->

- [ ] 正常系のテスト
- [ ] エラー系のテスト  
- [ ] 境界値のテスト
- [ ] パフォーマンステスト（必要に応じて）

## チェックリスト

### 実装

- [ ] コードが正常に動作する
- [ ] エラーハンドリングが適切に実装されている
- [ ] パフォーマンスへの影響を考慮している
- [ ] メモリリークやリソースリークがない

### テスト

- [ ] 新機能に対するテストを追加した
- [ ] 既存のテストが全て通る
- [ ] テストケースが適切にエラーケースをカバーしている

### ドキュメント

- [ ] 必要に応じてCLAUDE.mdを更新した
- [ ] 新しいAPIがあればドキュメントコメントを追加した

### R7RS準拠

- [ ] R7RS仕様に準拠している（該当する場合）
- [ ] SRFI仕様に準拠している（該当する場合）

## パフォーマンス影響

<!-- パフォーマンスへの影響があれば記載してください -->

- [ ] パフォーマンス影響なし
- [ ] パフォーマンス向上: XX%改善
- [ ] パフォーマンス低下: XX%低下（理由: XXX）

## 破壊的変更

<!-- 既存APIに影響する変更があれば記載してください -->

- [ ] 破壊的変更なし
- [ ] 破壊的変更あり（詳細: XXX）

## スクリーンショット・デモ

<!-- 新機能のデモがあれば貼り付けてください -->

```scheme
;; 使用例
(example-code-here)
```

## 追加情報

<!-- その他、レビュアーに伝えたい情報があれば記載してください -->

### 設計判断

<!-- 重要な設計判断があれば説明してください -->

### 既知の制限

<!-- 既知の制限や今後の改善点があれば記載してください -->

### 参考資料

<!-- 実装時に参考にした資料があれば記載してください -->

- [R7RS仕様書](https://small.r7rs.org/)
- [SRFI文書](https://srfi.schemers.org/)