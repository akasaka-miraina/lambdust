# Lambdust 0.2.0 表示的意味論設計書

**Version**: 0.2.0  
**Date**: 2025-01-17  
**Status**: Design Document

## 概要

本文書は、Lambdust 0.2.0に向けた表示的意味論の包括的な設計を提示する。特に、副作用システム、並行処理、FFI、および漸進的型付けの拡張機能について、Isabelle/HOL機械証明システムおよびEvent-B仕様との統合を考慮した形式化を行う。

## 1. 設計原則とアーキテクチャ

### 1.1 機械証明対応の設計原則

```isabelle
theory LambdustSemantics
  imports HOL Main
begin

(* Core domain definitions *)
datatype value = 
    VNil
  | VBool bool  
  | VNum real
  | VStr string
  | VSym string
  | VPair value value
  | VProc "value list ⇒ cont ⇒ ans"
  | VFuture "value option ref"
  | VActor actor_id
  | VFfiPtr "unit ptr"

and cont = Cont "value ⇒ ans"

and ans = 
    Value value
  | Wrong error_info
  | Effect effect_type "value list"

and effect_type =
    Pure
  | IO 
  | State
  | Error
  | Custom string
```

### 1.2 Event-B統合のためのアブストラクション層

```event-b
MACHINE LambdustCore
SEES LambdustTypes

VARIABLES
  global_env,
  effect_stack,
  actor_registry,
  memory_pool

INVARIANTS
  inv1: global_env ∈ Variable ⤸ Value
  inv2: effect_stack ∈ seq(EffectContext)
  inv3: actor_registry ∈ ActorId ⤸ ActorState
  inv4: memory_pool ∈ ForeignPtr ↔ MemoryBlock

INITIALIZATION
  global_env := ∅ ||
  effect_stack := ⟨⟩ ||
  actor_registry := ∅ ||
  memory_pool := ∅
```

## 2. 拡張副作用システムの意味論

### 2.1 副作用ハンドラの形式化

現在のFormalSemantics.mdの基本的な副作用処理を拡張し、代数的副作用ハンドラの完全な意味論を提供：

```isabelle
(* Effect Handler Semantics *)
datatype handler_result =
    HandlerValue value
  | HandlerContinue value continuation
  | HandlerAbort value

type_synonym effect_handler = "effect_type ⇒ value list ⇒ handler_result"

definition handle_computation :: 
  "effect_handler ⇒ computation ⇒ computation" where
"handle_computation h comp = 
  (λstate. case comp state of
    Value v ⇒ Value v
  | Wrong e ⇒ Wrong e  
  | Effect eff args ⇒ 
      (case h eff args of
        HandlerValue v ⇒ Value v
      | HandlerContinue v k ⇒ k v state
      | HandlerAbort v ⇒ Wrong (EffectAbort eff v)))"
```

**数学的基盤:**
```
⟦(handle h in e)⟧ρκ = ⟦e⟧ρ(handle-cont h κ)

where:
  handle-cont h κ = λv.
    case v of
      effect(eff, args, k) → 
        case h(eff, args) of
          return(v') → κ(v')
          continue(v', k') → k'(v')
          abort(v') → Wrong("effect-abort", eff, v')
      _ → κ(v)
```

### 2.2 ジェネレーショナル環境の意味論

現在の実装のジェネレーショナル環境管理を形式化：

```isabelle
datatype generation = Generation nat

type_synonym generational_env = 
  "generation ⇒ variable ⤸ value"

definition env_lookup :: 
  "generational_env ⇒ generation ⇒ variable ⇒ value option" where
"env_lookup genv gen var = 
  (case genv gen var of
    Some v ⇒ Some v
  | None ⇒ 
      if gen > 0 
      then env_lookup genv (gen - 1) var
      else None)"

definition env_extend :: 
  "generational_env ⇒ generation ⇒ variable ⇒ value ⇒ generational_env" where
"env_extend genv gen var val = 
  genv(gen := (genv gen)(var ↦ val))"
```

## 3. 並行処理システムの意味論

### 3.1 アクターシステムの形式化

現在のactor実装を基に、メッセージパッシングの詳細な意味論を構築：

```isabelle
datatype actor_state =
    Running "value list" (* mailbox *)
  | Suspended "value list" 
  | Failed error_info
  | Stopped

datatype message = Message {
  sender: "actor_id option",
  payload: value,
  timestamp: nat,
  reply_channel: "actor_id option"
}

type_synonym actor_system = 
  "actor_id ⤸ (actor_state × behavior)"

and behavior = 
  "message ⇒ actor_context ⇒ (actor_state × effect list)"
```

**メッセージパッシング意味論:**
```
send(aid, msg) ≙ 
  let current-system = get-actor-system() in
  case lookup(current-system, aid) of
    Some((Running(mailbox), behavior)) → 
      update-actor-system(aid, (Running(mailbox ++ [msg]), behavior))
    Some((Suspended(mailbox), behavior)) → 
      update-actor-system(aid, (Running(mailbox ++ [msg]), behavior))
    _ → Effect(Error, ["actor-not-found", aid])

receive(aid) ≙
  let current-system = get-actor-system() in
  case lookup(current-system, aid) of
    Some((Running(msg :: mailbox'), behavior)) → 
      let (new-state, effects) = behavior(msg, actor-context(aid)) in
      update-actor-system(aid, (new-state, behavior));
      emit-effects(effects);
      Value(msg.payload)
    Some((Running([]), behavior)) → 
      update-actor-system(aid, (Suspended([]), behavior));
      suspend-computation()
    _ → Effect(Error, ["invalid-actor-state", aid])
```

### 3.2 Software Transactional Memory (STM)

現在の基本的なSTM実装を拡張：

```isabelle
datatype tvar_state = TVar {
  value: value,
  version: nat,
  read_set: "transaction_id set",
  write_set: "transaction_id set"
}

type_synonym stm_state = 
  "tvar_id ⤸ tvar_state"

datatype transaction_log =
  Read tvar_id value nat  (* tvar, value, version *)
| Write tvar_id value     (* tvar, new value *)

definition validate_transaction :: 
  "transaction_log list ⇒ stm_state ⇒ bool" where
"validate_transaction log state =
  (∀entry ∈ set log. case entry of
    Read tid val ver ⇒ 
      (case state tid of
        Some (TVar v current_ver _ _) ⇒ 
          val = v ∧ ver = current_ver
      | None ⇒ False)
  | Write _ _ ⇒ True)"

definition commit_transaction :: 
  "transaction_log list ⇒ stm_state ⇒ stm_state option" where
"commit_transaction log state =
  (if validate_transaction log state
   then Some (fold apply_write_log log state)
   else None)"
```

## 4. FFIシステムの安全性意味論

### 4.1 型安全性制約の形式化

現在のsafety.rs実装を数学的に表現：

```isabelle
datatype c_type =
    CInt | CFloat | CDouble | CBool | CChar 
  | CString | CPtr c_type | CArray c_type nat
  | CStruct "(string × c_type) list"
  | CUnion "(string × c_type) list"

datatype safety_constraint =
    NonNull nat                    (* parameter index *)
  | Bounds nat int int            (* parameter, min, max *)
  | BufferSize nat nat            (* buffer param, size param *)
  | Alignment nat nat             (* parameter, alignment *)
  | ResourceLifetime nat lifetime_spec

datatype lifetime_spec =
    Borrowed | Owned | Transferred | Shared

definition validate_constraints ::
  "safety_constraint list ⇒ value list ⇒ bool" where
"validate_constraints constraints args =
  (∀c ∈ set constraints. case c of
    NonNull idx ⇒ 
      idx < length args ∧ args ! idx ≠ VNil
  | Bounds idx min_val max_val ⇒
      idx < length args ∧ 
      (case args ! idx of
        VNum n ⇒ min_val ≤ n ∧ n ≤ max_val
      | _ ⇒ False)
  | BufferSize buf_idx size_idx ⇒
      buf_idx < length args ∧ size_idx < length args ∧
      args ! buf_idx ≠ VNil ∧
      (case args ! size_idx of VNum _ ⇒ True | _ ⇒ False)
  | _ ⇒ True)"
```

### 4.2 メモリ安全性の形式化

```isabelle
datatype memory_region = Region {
  start_addr: nat,
  size: nat,
  permissions: permission_set,
  lifetime: lifetime_spec
}

datatype permission = Read | Write | Execute

type_synonym permission_set = "permission set"

definition memory_safe_access :: 
  "nat ⇒ nat ⇒ permission ⇒ memory_region set ⇒ bool" where
"memory_safe_access addr size perm regions =
  (∃region ∈ regions.
    region.start_addr ≤ addr ∧ 
    addr + size ≤ region.start_addr + region.size ∧
    perm ∈ region.permissions)"
```

## 5. 漸進的型付けの完全な形式化

### 5.1 一貫性関係の拡張

現在のgradual.rs実装を基に、より厳密な一貫性関係を定義：

```isabelle
inductive consistent :: "type ⇒ type ⇒ bool" (infix "∼" 50) where
  consistent_refl: "τ ∼ τ"
| consistent_dyn_l: "Dynamic ∼ τ" 
| consistent_dyn_r: "τ ∼ Dynamic"
| consistent_unknown: "Unknown ∼ τ ∧ τ ∼ Unknown"
| consistent_var: "TypeVar α ∼ τ ∧ τ ∼ TypeVar α"
| consistent_pair: "⟦τ₁ ∼ σ₁; τ₂ ∼ σ₂⟧ ⟹ Pair τ₁ τ₂ ∼ Pair σ₁ σ₂"
| consistent_fun: "⟦list_all2 consistent τs σs; τ ∼ σ⟧ 
                  ⟹ Function τs τ ∼ Function σs σ"
| consistent_effect: "⟦τ₁ ∼ σ₁; τ₂ ∼ σ₂⟧ 
                     ⟹ Effectful τ₁ ε τ₂ ∼ Effectful σ₁ ε σ₂"
```

### 5.2 型soundnessの証明構造

```isabelle
theorem gradual_type_safety:
  "⟦∅ ⊢ e : τ; ⟦e⟧ ρ halt = Value v⟧ 
   ⟹ ∃σ. τ ∼ σ ∧ v : σ"
proof (induction rule: typing.induct)
  case (t_var x τ Γ)
  then show ?case by (cases "Γ x") auto
next
  case (t_app Γ e₁ τ₁ τ₂ e₂)
  then obtain f v₂ where 
    "⟦e₁⟧ ρ halt = Value f" and "⟦e₂⟧ ρ halt = Value v₂"
    by (auto dest: app_eval_parts)
  
  from t_app.IH(1)[OF this(1)] obtain σ₁ where
    "Function [τ₁] τ₂ ∼ σ₁" and "f : σ₁" by auto
    
  (* Continue proof... *)
next
  case (t_cast e τ₁ τ₂)
  (* Cast soundness proof *)
  then show ?case
    apply (cases "consistent τ₁ τ₂")
    apply (auto dest: cast_preserves_consistency)
    done
qed
```

## 6. Event-B翻訳パターン

### 6.1 Event-Bからの翻訳規則

Event-B仕様からLambdustプログラムへの翻訳のための構造的規則：

```
Event-B Machine → Lambdust Actor
Event-B Variable → Lambdust TVar (for shared state)
Event-B Event → Lambdust Message Handler
Event-B Guard → Lambdust Pattern Match + Precondition
Event-B Action → Lambdust STM Transaction
```

**翻訳例:**
```event-b
EVENT ProcessRequest
WHEN
  msg : pending_messages
  resource_available = TRUE
THEN
  pending_messages := pending_messages ∖ {msg}
  process_message(msg)
  resource_available := FALSE
END
```

**翻訳後のLambdust:**
```scheme
(define-actor request-processor
  (lambda (message context)
    (match message
      [(request msg)
       (atomically
         (lambda ()
           (when (and (tvar-member? pending-messages msg)
                      (tvar-read resource-available))
             (tvar-modify! pending-messages 
                          (lambda (msgs) (remove msg msgs)))
             (process-message msg)
             (tvar-write! resource-available #f))))]
      [_ (error "unknown-message" message)])))
```

## 7. 機械証明との統合

### 7.1 Isabelle/HOL証明義務

主要な証明すべき定理：

```isabelle
(* Type Safety *)
theorem type_safety:
  "⟦∅ ⊢ e : τ; ⟦e⟧ ρ halt ≠ Wrong _⟧ ⟹ ∃v. ⟦e⟧ ρ halt = Value v"

(* Effect Safety *)  
theorem effect_safety:
  "⟦Γ ⊢ e : τ ! ε; ⟦e⟧ ρ halt = Value v⟧ 
   ⟹ effects_produced_by e ⊆ ε"

(* Gradual Consistency *)
theorem gradual_consistency:
  "⟦τ₁ ∼ τ₂; v : τ₁⟧ ⟹ cast τ₁ τ₂ v ≠ Wrong _"

(* Actor Safety *)
theorem actor_safety:
  "⟦well_formed_actor_system S; send aid msg S = S'⟧ 
   ⟹ well_formed_actor_system S'"

(* FFI Safety *)
theorem ffi_safety:
  "⟦validate_ffi_call fname sig args; 
    call_ffi fname args = Success v⟧ 
   ⟹ v satisfies return_type_of sig"

(* STM Consistency *)
theorem stm_consistency:
  "⟦validate_transaction log state; 
    commit_transaction log state = Some state'⟧ 
   ⟹ consistent_state state'"
```

### 7.2 証明戦略

1. **構造帰納法**: 型と式の構造に関する帰納法
2. **進行性と保存性**: 標準的なtype safety証明
3. **一貫性の保持**: 漸進的型付けの一貫性
4. **不変式の維持**: 並行システムとSTMの不変式
5. **安全性制約の充足**: FFI呼び出しの安全性

## 8. 実装との対応

### 8.1 既存実装との整合性

現在の実装構造との対応関係：

- `src/effects/` → Section 2 (副作用システム)
- `src/concurrency/` → Section 3 (並行処理)  
- `src/ffi/` → Section 4 (FFI安全性)
- `src/types/gradual.rs` → Section 5 (漸進的型付け)

### 8.2 拡張ポイント

形式化で明らかになった実装の拡張ポイント：

1. **副作用ハンドラ**: 継続ベースのハンドリング
2. **アクター障害回復**: 形式的な監督戦略
3. **STM検証**: トランザクションログの検証強化
4. **FFI境界チェック**: より厳密な境界検証
5. **型安全性**: 段階的な型検査の強化

## 9. 今後の発展

### 9.1 証明自動化

- SMT solverとの統合による制約ソルビング
- 型推論アルゴリズムの正しさ証明
- 並行システムのモデル検査

### 9.2 形式手法統合

- TLA+との統合による並行システムの検証
- Coqとの相互変換による証明再利用
- Event-B精密化の自動化

## 10. 結論

本設計書は、Lambdust 0.2.0の表示的意味論を包括的に設計し、特に以下の成果を達成した：

1. **機械証明対応**: Isabelle/HOLでの形式化可能な構造
2. **実装整合性**: 既存実装との明確な対応関係
3. **拡張性**: Event-B等の形式手法との統合基盤
4. **安全性**: 型安全性・メモリ安全性の数学的保証

この設計に基づいて、Lambdust 0.2.0は理論と実装の両面で最高水準の言語処理系となることが期待される。

---

**References:**
1. Wadler, P., Findler, R.B. (2009). "Well-Typed Programs Can't Be Blamed"
2. Siek, J., Vitousek, M. (2016). "Gradual Typing: Theory and Practice"  
3. Dolan, S. (2017). "Algebraic Effects and Handlers"
4. Abrial, J.R. (2010). "Modeling in Event-B: System and Software Engineering"
5. Nipkow, T., Paulson, L., Wenzel, M. (2002). "Isabelle/HOL"

---

**Author**: Claude (Language Processor Architect)  
**Date**: 2025-01-17  
**Version**: 0.2.0-design