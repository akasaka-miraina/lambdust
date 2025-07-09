-- R7RS Scheme Core Semantics
-- 基本データ型、環境、評価関数の形式的定義

module R7RS.Core where

open import Agda.Builtin.String
open import Agda.Builtin.Nat
open import Agda.Builtin.Bool
open import Agda.Builtin.List
open import Agda.Builtin.Equality

-- 基本Scheme値
data SchemeValue : Set where
  SNumber   : Nat → SchemeValue
  SBoolean  : Bool → SchemeValue  
  SSymbol   : String → SchemeValue
  SPair     : SchemeValue → SchemeValue → SchemeValue
  SNil      : SchemeValue
  SClosure  : Env → List String → SchemeExpr → SchemeValue

-- 環境：変数名から値への写像
data Env : Set where
  EmptyEnv : Env
  ExtendEnv : String → SchemeValue → Env → Env

-- Scheme式
data SchemeExpr : Set where
  Literal     : SchemeValue → SchemeExpr
  Variable    : String → SchemeExpr
  Application : SchemeExpr → List SchemeExpr → SchemeExpr
  Lambda      : List String → SchemeExpr → SchemeExpr
  If          : SchemeExpr → SchemeExpr → SchemeExpr → SchemeExpr
  Begin       : List SchemeExpr → SchemeExpr

-- 環境での変数探索
lookup : String → Env → Maybe SchemeValue
lookup x EmptyEnv = nothing
lookup x (ExtendEnv y v env) with x == y
... | true  = just v
... | false = lookup x env

-- 環境拡張
extend : List String → List SchemeValue → Env → Env
extend [] [] env = env
extend (x ∷ xs) (v ∷ vs) env = extend xs vs (ExtendEnv x v env)
extend _ _ env = env -- mismatched lengths

-- 基本評価関数（仮）
-- TODO: 完全なCPS意味論での実装
eval : SchemeExpr → Env → Maybe SchemeValue
eval (Literal v) env = just v
eval (Variable x) env = lookup x env
eval (Lambda params body) env = just (SClosure env params body)
eval (Application f args) env = {!!} -- TODO: 関数適用
eval (If test then else) env = {!!}   -- TODO: 条件分岐  
eval (Begin exprs) env = {!!}         -- TODO: 順次実行

-- 最適化の意味論的等価性定義
_≈_ : SchemeExpr → SchemeExpr → Set
e₁ ≈ e₂ = ∀ (env : Env) → eval e₁ env ≡ eval e₂ env

-- 最適化の正当性証明の型
OptimizationCorrect : (SchemeExpr → SchemeExpr) → Set
OptimizationCorrect opt = ∀ (e : SchemeExpr) → e ≈ opt e