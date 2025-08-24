# NaN Boxing Value型設計仕様書

## 概要

NaN Boxingは、IEEE 754 double-precision floating pointのNaN値のビットパターンを活用して、複数のLambdust Value型を単一の64bit値に圧縮する最適化手法です。

## 60%メモリ削減目標の理論的根拠

### 現在のValue enumメモリ使用量
```rust
enum Value {
    // 各variant + discriminant = ~24-32 bytes
    Number(f64),              // 8 bytes + header
    Boolean(bool),            // 1 byte + header + padding
    String(Arc<String>),      // 8 bytes pointer + header
    Vector(Arc<Vec<Value>>),  // 8 bytes pointer + header
    // ... 40+ variants
}
```
**推定現在サイズ**: 24-32 bytes per Value

### NaN Boxing後のメモリ使用量
```rust
struct NanBoxedValue(u64);  // 8 bytes ONLY
```
**目標サイズ**: 8 bytes per Value
**理論削減率**: 66-75% (目標60%を超過)

## IEEE 754 NaN Boxing ビットレイアウト

### IEEE 754 Double-Precision構造
```
Bit:  63    62-52      51-0
     [ S ] [Exponent] [Mantissa]
Sign: 1bit
Exp:  11bits (all 1s = NaN when mantissa ≠ 0)
Man:  52bits
```

### NaN Boxing戦略
```
NaN検出: Exponent = 0x7FF (all 1s)
利用可能ビット: 52bits (mantissa) + 1bit (sign) = 53bits

53bits分割案:
- Type Tag: 4bits (16種類の型対応)
- Payload:  49bits (データ格納用)
```

### 型エンコーディング設計

#### インライン値 (direct encoding)
```
0x0: False (boolean)
0x1: True (boolean)
0x2: Nil (empty list)
0x3: Unspecified
0x4: EOF object
0x5: Small integers (-2^24 to 2^24-1)
0x6: Characters (Unicode code point)
0x7: Symbols (interned symbol ID)
```

#### ポインタ値 (pointer encoding)
```
0x8: String pointer (Arc<String>)
0x9: Vector pointer (Arc<Vec<Value>>)
0xA: Procedure pointer
0xB: Port pointer
0xC: Promise pointer
0xD: Record pointer
0xE: Advanced containers
0xF: Reserved/Extension
```

## 実装戦略

### Phase 1: 基本型の最適化
```rust
#[repr(C)]
pub struct NanBoxedValue(u64);

impl NanBoxedValue {
    // Immediate values
    const FALSE: u64 = 0x7FF0_0000_0000_0000;
    const TRUE: u64  = 0x7FF0_0000_0000_0001;
    const NIL: u64   = 0x7FF0_0000_0000_0002;
    
    // Type masks
    const TYPE_MASK: u64 = 0x000F_0000_0000_0000;
    const PAYLOAD_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;
    
    pub fn from_bool(b: bool) -> Self {
        Self(if b { Self::TRUE } else { Self::FALSE })
    }
    
    pub fn is_boolean(&self) -> bool {
        (self.0 & 0xFFFF_FFFF_FFFF_FFFE) == Self::FALSE
    }
}
```

### Phase 2: 互換性レイヤー
```rust
impl From<Value> for NanBoxedValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Boolean(b) => Self::from_bool(b),
            Value::Number(n) => {
                if n.is_nan() {
                    // Handle NaN numbers specially
                    Self::from_heap_number(n)
                } else {
                    // Direct IEEE 754 representation
                    Self(n.to_bits())
                }
            }
            Value::String(s) => Self::from_pointer(0x8, s.as_ptr()),
            // ...
        }
    }
}

impl From<NanBoxedValue> for Value {
    fn from(boxed: NanBoxedValue) -> Self {
        if boxed.is_number() {
            Value::Number(f64::from_bits(boxed.0))
        } else if boxed.is_boolean() {
            Value::Boolean(boxed.0 == NanBoxedValue::TRUE)
        }
        // ...
    }
}
```

### Phase 3: パフォーマンス最適化

#### インライン型チェック
```rust
impl NanBoxedValue {
    #[inline(always)]
    pub fn is_number(&self) -> bool {
        (self.0 & 0x7FF0_0000_0000_0000) != 0x7FF0_0000_0000_0000
    }
    
    #[inline(always)]
    pub fn is_immediate(&self) -> bool {
        self.is_number() || ((self.0 & 0xFFF0_0000_0000_0000) == 0x7FF0_0000_0000_0000)
    }
}
```

#### SIMD最適化対応
```rust
// Vector operations can work on [NanBoxedValue; 4] directly
#[repr(C)]
struct NanBoxedVector([NanBoxedValue; 4]);
```

## 型安全性保証

### Rust型システム活用
```rust
pub struct NanBoxedValue(u64);

// No public constructor for raw u64
impl NanBoxedValue {
    // Type-safe constructors only
    pub fn from_bool(b: bool) -> Self { /* ... */ }
    pub fn from_number(n: f64) -> Self { /* ... */ }
    
    // Type-safe extractors
    pub fn as_bool(&self) -> Option<bool> { /* ... */ }
    pub fn as_number(&self) -> Option<f64> { /* ... */ }
}
```

### デバッグ支援
```rust
impl Debug for NanBoxedValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_number() {
            write!(f, "Number({})", self.as_number().unwrap())
        } else if self.is_boolean() {
            write!(f, "Boolean({})", self.as_bool().unwrap())
        } else {
            write!(f, "NanBoxed(0x{:016x})", self.0)
        }
    }
}
```

## 互換性戦略

### 段階的移行
1. **既存Valueと並行稼働**: 初期は両システム共存
2. **選択的適用**: パフォーマンス重要箇所から適用
3. **完全移行**: 検証完了後、全面切り替え

### フォールバック機構
```rust
enum ValueStorage {
    NanBoxed(NanBoxedValue),
    Fallback(Box<Value>),  // Complex types
}
```

## パフォーマンス特性

### 期待される効果
- **メモリ使用量**: 60-75%削減
- **キャッシュ効率**: 大幅改善 (8 bytes vs 32 bytes)
- **コピー性能**: 高速化 (単純な64bit copy)
- **型チェック**: 高速化 (ビット操作のみ)

### ベンチマーク計画
- メモリ使用量測定
- Value creation/destruction速度
- 型チェック速度
- 実際のScheme プログラム性能

## リスク評価

### 技術的リスク
- **ポインタアライメント**: 64bit境界要求
- **GC統合**: 既存GCシステムとの整合性
- **デバッグ困難性**: ビットレベル操作の複雑性

### 緩和策
- 徹底的な単体テスト
- 段階的導入
- 詳細なドキュメント化
- デバッグ支援ツール充実

## 実装マイルストーン

1. **Week 1**: 基本NanBoxedValue構造体実装
2. **Week 2**: 基本型 (bool, number, nil) 対応
3. **Week 3**: 互換性レイヤー実装
4. **Week 4**: パフォーマンステスト＆最適化

この設計により、60%メモリ削減目標の達成と、同時に実行性能の向上も期待できます。