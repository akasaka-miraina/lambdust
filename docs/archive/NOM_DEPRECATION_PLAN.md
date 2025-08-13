# nom_compat.rs 段階的廃止計画

**策定日**: 2025年8月12日  
**目標**: nom_compat.rsを完全に削除し、純粋な内製パーサーシステムに移行  
**原則**: 段階的・安全な移行により既存機能を破綻させない

## 📊 現状分析

### nom_compat.rsの役割
- nom APIとの互換性レイヤー
- 段階的移行のための一時的な橋渡し
- 既存コードの保護

### 問題点
- **技術的負債**: 過去の名残としての複雑性
- **保守コスト**: 二重実装による維持負荷
- **コードの肥大化**: 不要な互換レイヤー
- **パフォーマンス**: 余分な抽象化による性能低下

## 🎯 段階的廃止戦略

### Phase A: 依存関係分析と代替準備 (2-3週間)
**目標**: nom_compatの使用状況を完全把握し、代替手段を準備

#### A.1: 使用状況の全数調査
```bash
# nom_compat使用箇所の特定
grep -r "nom_compat" src/
grep -r "use.*nom_api" src/
grep -r "HybridParser" src/
grep -r "UnifiedParser" src/
```

#### A.2: 代替API設計
```rust
// 現在: nom_compat経由
use lambdust::parser::combinators::nom_api::tag;

// 移行後: 直接内製API
use lambdust::parser::combinators::tag;
```

#### A.3: 移行ガイドライン作成
- 関数対応表の作成
- 自動変換スクリプトの準備
- コード変更パターンの文書化

### Phase B: 段階的機能移行 (3-4週間)
**目標**: nom_compatの機能を順次内製APIに統合

#### B.1: 基本パーサー移行
```rust
// 移行対象
nom_api::tag()          → primitive::tag()
nom_api::char()         → primitive::char()  
nom_api::digit1()       → primitive::digits()
nom_api::alpha1()       → primitive::letters()
```

#### B.2: コンビネータ移行
```rust  
// 移行対象
nom_api::many0()        → combinator::many()
nom_api::many1()        → combinator::many1()
nom_api::opt()          → combinator::optional()
nom_api::alt2()         → combinator::or()
```

#### B.3: 高級機能移行
```rust
// 移行対象
HybridParser           → 削除（内製パーサーに統一）
UnifiedParser          → 削除（Feature Flag削除）
MigrationChecklist     → 削除（移行完了後不要）
```

### Phase C: nom_compat完全削除 (1-2週間)  
**目標**: nom_compat.rsファイルとその依存を完全除去

#### C.1: ファイル削除準備
- 全使用箇所の移行完了確認
- テストスイートでの動作確認
- 性能ベンチマークでの確認

#### C.2: 段階的削除実行
```bash
# 1. nom_compatモジュールの無効化
# src/parser/combinators/mod.rs
// pub mod nom_compat;  // コメントアウト

# 2. import削除の自動化
find src/ -name "*.rs" -exec sed -i 's/use.*nom_compat.*;//g' {} \;

# 3. ファイル物理削除
rm src/parser/combinators/nom_compat.rs
rm src/parser/combinators/feature_parser.rs
```

#### C.3: 最終クリーンアップ
- 関連するFeature Flagの削除
- Cargo.tomlからinternal-parserフィーチャー削除
- 文書化の更新

## 🚀 新しいパーサーAPI設計

### 統一されたインターフェース
```rust
// 新しい統一API設計
pub mod lambdust::parser {
    // 基本パーサープリミティブ
    pub use combinators::primitive::*;
    
    // 高級コンビネータ
    pub use combinators::combinator::*;
    
    // Scheme特化パーサー
    pub use combinators::scheme::*;
}

// 使用例
use lambdust::parser::{tag, char, many, optional};

let parser = many(char('a'));
let result = parser.parse(input);
```

### API簡素化
```rust
// Before: nom_compat経由の複雑なAPI
let parser = UnifiedParser::new()
    .with_backend(ParserBackend::Internal)
    .parse_with(nom_api::tag("hello"));

// After: シンプルな直接API
let parser = tag("hello");
let result = parser.parse(input);
```

## 📈 期待される効果

### バイナリサイズ削減
```
nom_compat.rs削除前:     +120KB (互換レイヤー)
feature_parser.rs削除:   +80KB  (Feature Flag)
関連インフラ削除:        +50KB  (ヘルパー等)
--------------------------------
削除後の削減効果:        -250KB (約4.5%削減)
```

### パフォーマンス向上
```
互換レイヤー除去:        +15% (間接呼び出し削減)
Feature Flag判定削除:    +5%  (分岐削減)  
API統一による最適化:     +10% (特化最適化)
--------------------------------
総合性能向上:           +30% (予想)
```

### 開発・保守性向上
```
コード複雑性削減:        -40% (二重実装削除)
テストケース削減:        -25% (互換性テスト不要)
文書化負荷軽減:          -35% (単一API)  
新機能開発速度:          +50% (設計統一)
```

## 🛡️ リスク管理

### 潜在的リスク
1. **既存コードの破綻**: nom_compat依存の見落とし
2. **API変更の影響**: 微妙な動作差異
3. **性能劣化**: 予想外のボトルネック
4. **テスト不備**: 移行漏れの検出失敗

### 対策
1. **段階的移行**: 1つずつ確実に移行
2. **包括的テスト**: 各段階での動作確認
3. **自動検証**: CI/CDでの継続確認
4. **ロールバック準備**: 問題時の即座復旧

## ⏱️ 実行スケジュール

### タイムライン
```
Week 1-2:  Phase A.1-A.2 (分析・設計)
Week 3:    Phase A.3     (ガイドライン)
Week 4-5:  Phase B.1     (基本パーサー移行)  
Week 6-7:  Phase B.2-B.3 (高級機能移行)
Week 8:    Phase C.1-C.2 (完全削除)
Week 9:    Phase C.3     (最終クリーンアップ)
```

### マイルストーン
- **Week 3**: 移行計画確定
- **Week 5**: 基本機能移行完了  
- **Week 7**: 全機能移行完了
- **Week 9**: nom_compat完全削除

## 🎯 成功指標

### 定量的指標
- **コンパイルエラー**: 0個維持
- **テスト成功率**: 100%維持  
- **バイナリサイズ**: 250KB削減達成
- **性能向上**: 30%向上達成

### 定性的指標
- **コード可読性**: API統一による改善
- **開発効率**: 単純化による向上
- **保守性**: 技術的負債削減
- **拡張性**: 純粋設計による改善

## 📋 実行チェックリスト

### Phase A チェックリスト
- [ ] nom_compat使用箇所の全数調査完了
- [ ] 代替API設計完了
- [ ] 自動変換スクリプト作成
- [ ] 移行ガイドライン文書化

### Phase B チェックリスト  
- [ ] 基本パーサー移行完了
- [ ] コンビネータ移行完了
- [ ] 高級機能移行完了
- [ ] 各段階でのテスト確認

### Phase C チェックリスト
- [ ] 全使用箇所移行確認
- [ ] nom_compat.rs物理削除
- [ ] feature_parser.rs削除
- [ ] 関連Feature Flag削除
- [ ] 文書更新完了

## 💡 提案

この計画に従って、nom_compat.rsを段階的に廃止することで：

1. **技術的負債の解消**: 過去の名残を完全除去
2. **純粋な内製システム**: 依存のない自立したパーサー
3. **性能・保守性向上**: シンプルで効率的な設計
4. **開発速度向上**: 統一されたAPIによる開発効率化

nom_compat.rsの廃止により、Lambdustは真の意味で「内製パーサーシステム」として完成します。

---

**次のアクション**: Phase Aから開始し、段階的かつ安全にnom_compatを除去していきます。