# Lambdust Formal Semantics

**Version**: 1.0.0  
**Date**: 2025-01-17  
**Status**: Living Document

This document provides the formal denotational semantics for Lambdust, covering R7RS-small, selected SRFIs, and Lambdust-specific extensions. The semantics are presented using mathematical notation and domain theory.

## Table of Contents

1. [Mathematical Foundations](#mathematical-foundations)
2. [Core Domains](#core-domains)
3. [R7RS-small Semantics](#r7rs-small-semantics)
4. [SRFI Extensions](#srfi-extensions)
5. [Lambdust Type System](#lambdust-type-system)
6. [Effect System](#effect-system)
7. [Concurrency Semantics](#concurrency-semantics)
8. [FFI Semantics](#ffi-semantics)
9. [Metaprogramming](#metaprogramming)
10. [Operational Correspondences](#operational-correspondences)

---

## Mathematical Foundations

### Basic Notation

- `⊥` denotes undefined/bottom value
- `D⊥ = D ∪ {⊥}` is the lifted domain D
- `f : A → B` denotes a total function from A to B  
- `f : A ⇀ B` denotes a partial function from A to B
- `[A → B]` denotes the continuous function space
- `A × B` denotes Cartesian product
- `A + B` denotes disjoint union
- `μX. F(X)` denotes least fixed point of F
- `⟦e⟧ρ` denotes the denotation of expression e in environment ρ

### Domain Equations

The semantic domains are defined by the following system of recursive domain equations:

```
Val = Bool + Num + Sym + Char + Str + Pair + Vec + Proc + Port + Cont
    + Record + Promise + Complex + Rational + Text + HashTable 
    + Ideque + PriorityQueue + OrderedSet + Future + Actor + FfiPtr

Proc = [Val* → Cont → Ans]
Cont = [Val → Ans]  
Env = Var ⇀ Val
Ans = Val + Wrong

Store = Loc ⇀ Val
State = Store × Eff*
Eff = IO | State | Alloc | Async | Meta
```

Where:
- `Bool = {#t, #f}`
- `Num = ℝ` (real numbers, including rationals and complex)
- `Sym` is the domain of symbols
- `Char` is the domain of Unicode characters
- `Str` is the domain of Unicode strings
- `Pair = Val × Val`
- `Vec = Val*` (sequences of values)
- `Wrong` represents runtime errors

---

## Core Domains

### 2.1 Values (Val)

The value domain includes all first-class Scheme values:

```
⟦#t⟧ = true ∈ Bool
⟦#f⟧ = false ∈ Bool
⟦n⟧ = n ∈ Num  (where n is a numeric literal)
⟦'sym⟧ = sym ∈ Sym
⟦#\c⟧ = c ∈ Char
⟦"str"⟧ = str ∈ Str
```

### 2.2 Environment (Env)

Environment maps variables to values:

```
initial-env : Env = {
  + ↦ λ(v₁, v₂). if v₁, v₂ ∈ Num then v₁ + v₂ else Wrong,
  - ↦ λ(v₁, v₂). if v₁, v₂ ∈ Num then v₁ - v₂ else Wrong,
  cons ↦ λ(v₁, v₂). (v₁, v₂) ∈ Pair,
  car ↦ λ(v). if v = (v₁, v₂) then v₁ else Wrong,
  cdr ↦ λ(v). if v = (v₁, v₂) then v₂ else Wrong,
  ...
}
```

### 2.3 Continuations (Cont)

Continuations represent "the rest of the computation":

```
halt : Cont = λv. v
cons₁ : Val → Cont → Cont = λv₂.λκ.λv₁. κ((v₁, v₂))
cons₂ : Cont → Cont = λκ.λv₂. ⟦cons⟧ [v₂, κ]
```

---

## R7RS-small Semantics

### 3.1 Expression Evaluation

The semantic function `⟦·⟧ : Exp → Env → Cont → Ans`:

#### Variables
```
⟦x⟧ρκ = if x ∈ dom(ρ) then κ(ρ(x)) else Wrong
```

#### Literals
```
⟦#t⟧ρκ = κ(true)
⟦#f⟧ρκ = κ(false)  
⟦n⟧ρκ = κ(n)
⟦'datum⟧ρκ = κ(⟦datum⟧)
```

#### Conditional
```
⟦(if e₁ e₂ e₃)⟧ρκ = ⟦e₁⟧ρ(λv. if v ≠ false then ⟦e₂⟧ρκ else ⟦e₃⟧ρκ)
```

#### Lambda Abstraction
```
⟦(lambda (x₁ ... xₙ) body)⟧ρκ = κ(λ(v₁,...,vₙ).λκ'. ⟦body⟧ρ[x₁↦v₁,...,xₙ↦vₙ]κ')
```

#### Application
```
⟦(e₀ e₁ ... eₙ)⟧ρκ = ⟦e₀⟧ρ(λf. ⟦e₁⟧ρ(λv₁. ... ⟦eₙ⟧ρ(λvₙ. f(v₁,...,vₙ)κ)))
```

#### Assignment
```
⟦(set! x e)⟧ρκ = ⟦e⟧ρ(λv. κ(unspecified)) where ρ := ρ[x ↦ v]
```

### 3.2 Definition Semantics

```
⟦(define x e)⟧ρκ = ⟦e⟧ρ(λv. κ(unspecified)) where ρ := ρ[x ↦ v]

⟦(define (f x₁ ... xₙ) body)⟧ρκ = 
  let proc = λ(v₁,...,vₙ).λκ'. ⟦body⟧ρ[f↦proc,x₁↦v₁,...,xₙ↦vₙ]κ'
  in κ(unspecified) where ρ := ρ[f ↦ proc]
```

### 3.3 Control Flow

#### Sequence
```
⟦(begin e₁ ... eₙ)⟧ρκ = ⟦e₁⟧ρ(λv₁. ... ⟦eₙ⟧ρκ)
```

#### Let Binding
```
⟦(let ((x₁ e₁) ... (xₙ eₙ)) body)⟧ρκ = 
  ⟦e₁⟧ρ(λv₁. ... ⟦eₙ⟧ρ(λvₙ. ⟦body⟧ρ[x₁↦v₁,...,xₙ↦vₙ]κ))
```

#### Recursive Let
```
⟦(letrec ((x₁ e₁) ... (xₙ eₙ)) body)⟧ρκ = 
  let ρ' = ρ[x₁↦⊥,...,xₙ↦⊥]
      fix = λρ''. ⟦e₁⟧ρ''(λv₁. ... ⟦eₙ⟧ρ''(λvₙ. 
              if ρ''[x₁↦v₁,...,xₙ↦vₙ] = ρ'' then ρ'' 
              else fix(ρ''[x₁↦v₁,...,xₙ↦vₙ])))
      ρ* = fix(ρ')
  in ⟦body⟧ρ*κ
```

### 3.4 Continuations

#### Call/cc
```
⟦(call/cc e)⟧ρκ = ⟦e⟧ρ(λf. f(λv.λ_. κ(v))κ)
```

#### Dynamic Wind
```
⟦(dynamic-wind before thunk after)⟧ρκ = 
  ⟦before⟧ρ(λ_. ⟦thunk⟧ρ(λv. ⟦after⟧ρ(λ_. κ(v))))
```

---

## SRFI Extensions

### 4.1 SRFI-1: List Library

#### Extended List Operations
```
⟦fold-left⟧ = λ(proc, init, list).
  letrec fold = λ(acc, ls). 
    if null?(ls) then acc
    else fold(proc(acc, car(ls)), cdr(ls))
  in fold(init, list)

⟦fold-right⟧ = λ(proc, init, list).
  if null?(list) then init
  else proc(car(list), fold-right(proc, init, cdr(list)))
```

#### List Comprehensions (Lambdust extension)
```
⟦(list-comp expr for x in list if pred)⟧ρκ =
  let comp = λ(result, ls). 
    if null?(ls) then κ(reverse(result))
    else let x-val = car(ls)
         in ⟦pred⟧ρ[x↦x-val](λpred-val.
              if pred-val ≠ false 
              then ⟦expr⟧ρ[x↦x-val](λexpr-val. 
                     comp(cons(expr-val, result), cdr(ls)))
              else comp(result, cdr(ls)))
  in comp(nil, list)
```

### 4.2 SRFI-9: Record Types

```
⟦(define-record-type name (constructor field₁ ... fieldₙ) predicate 
   (field₁ accessor₁ mutator₁) ...)⟧ρκ =
  
  let record-type = new-record-type(name, [field₁, ..., fieldₙ])
      ctor = λ(v₁,...,vₙ). make-record(record-type, [v₁,...,vₙ])
      pred = λ(v). is-record?(v, record-type)
      acc₁ = λ(v). if pred(v) then record-ref(v, 0) else Wrong
      mut₁ = λ(v,new). if pred(v) then record-set!(v, 0, new) else Wrong
      ...
  in κ(unspecified) where ρ := ρ[constructor ↦ ctor, predicate ↦ pred, 
                                 accessor₁ ↦ acc₁, mutator₁ ↦ mut₁, ...]
```

### 4.3 SRFI-39: Parameter Objects

```
Parameter = [Val → [Val → Val] → Val]

⟦(make-parameter init converter)⟧ρκ = 
  let cell = ref(converter(init))
  in κ(λ(arg, cont). 
        if arg = unspecified then !cell
        else let old = !cell
             in cell := converter(arg); 
                let result = cont(unspecified)
                in cell := old; result)
```

### 4.4 SRFI-125: Hash Tables

```
HashTable = Sym ⇀ Val

⟦(hash-table-ref ht key default)⟧ρκ =
  if key ∈ dom(ht) then κ(ht(key)) else κ(default)

⟦(hash-table-set! ht key value)⟧ρκ =
  κ(unspecified) where ht := ht[key ↦ value]
```

### 4.5 SRFI-135: Text Library

```
Text = UnicodeString × NormalizationForm

⟦(text-normalize text form)⟧ρκ = κ((normalize(text.string, form), form))

⟦(text=? text₁ text₂)⟧ρκ = 
  let (s₁, f₁) = text₁
      (s₂, f₂) = text₂
      norm₁ = normalize(s₁, NFC)
      norm₂ = normalize(s₂, NFC)
  in κ(norm₁ = norm₂)
```

---

## Lambdust Type System

### 5.1 Type Domains

```
Type = BasicType + FunctionType + ProductType + SumType + EffectfulType
     + ParametricType + ConstrainedType

BasicType = Bool | Num | Str | Sym | Unit
FunctionType = Type × Type  
ProductType = Type*
SumType = (Tag × Type)*
EffectfulType = Type × Effect* × Type
ParametricType = TypeVar × Type
ConstrainedType = TypeConstraint* × Type

TypeConstraint = TypeClass × Type
TypeClass = Eq | Ord | Show | Functor | Monad | ...
```

### 5.2 Type Inference

The type inference algorithm follows Hindley-Milner with extensions:

```
Γ ⊢ e : τ ⇝ C   (Constraint generation)
unify(C) = σ    (Constraint solving)
τ' = σ(τ)       (Type substitution)
```

#### Typing Rules

**Variables:**
```
x : τ ∈ Γ
─────────── (Var)
Γ ⊢ x : τ ⇝ ∅
```

**Lambda:**  
```
Γ, x : α ⊢ e : τ ⇝ C   (α fresh)
────────────────────────────── (Abs)
Γ ⊢ λx.e : α → τ ⇝ C
```

**Application:**
```
Γ ⊢ e₁ : τ₁ ⇝ C₁    Γ ⊢ e₂ : τ₂ ⇝ C₂   (α fresh)
──────────────────────────────────────────────── (App)
Γ ⊢ e₁ e₂ : α ⇝ C₁ ∪ C₂ ∪ {τ₁ ≐ τ₂ → α}
```

### 5.3 Gradual Typing

Lambdust supports gradual typing with dynamic type checking:

```
Dynamic = ⊤  (top type)

cast : Type → Type → Val → Val
cast(τ₁, τ₂, v) = 
  if compatible(τ₁, τ₂) then v
  else if τ₂ = Dynamic then v  
  else if τ₁ = Dynamic ∧ runtime-type(v) <: τ₂ then v
  else Wrong
```

---

## Effect System  

### 6.1 Effect Domains

```
Effect = Pure | IO | State | Alloc | Exn | Async | Meta | FFI

EffectSet = ℘(Effect)

Effect Ordering:
Pure ⊑ State ⊑ IO
Pure ⊑ Alloc ⊑ IO  
Pure ⊑ Async
Exn is incomparable with others
```

### 6.2 Effectful Computation

```
Computation τ ε = [State → (Val_τ × State × Effect*)⊥]

return : τ → Computation τ ∅
return(v) = λs. (v, s, [])

bind : Computation τ₁ ε₁ → (τ₁ → Computation τ₂ ε₂) → Computation τ₂ (ε₁ ∪ ε₂)  
bind(m, f) = λs. case m(s) of
  ⊥ → ⊥
  (v, s', eff₁) → case f(v)(s') of
    ⊥ → ⊥  
    (v', s'', eff₂) → (v', s'', eff₁ ++ eff₂)
```

### 6.3 Effect Handlers

```
handle : EffectHandler ε₁ τ₁ τ₂ → Computation τ₁ (ε₁ ∪ ε₂) → Computation τ₂ ε₂

EffectHandler ε τ₁ τ₂ = ∀e ∈ ε. (e → τ₁ → Computation τ₂ ε₂)
```

---

## Concurrency Semantics

### 7.1 Concurrent Processes

```
Process = ProcId × Code × LocalEnv
ProcId = ℕ
ProcessPool = ProcId ⇀ Process

Schedule : ProcessPool → ProcessPool
```

### 7.2 Futures and Promises

```
Future τ = Ref(FutureState τ)
FutureState τ = Pending | Resolved τ | Failed Error

⟦(future expr)⟧ρκ = 
  let fut = ref(Pending)
      pid = spawn(λ(). ⟦expr⟧ρ(λv. fut := Resolved(v)))
  in κ(fut)

⟦(force future)⟧ρκ = 
  case !future of
    Pending → block-until-resolved(future); force(future)
    Resolved(v) → κ(v)  
    Failed(e) → raise(e)
```

### 7.3 Actor Model

```
Actor = ActorId × Mailbox × Behavior
ActorId = ℕ  
Mailbox = Message*
Behavior = Message → Actor → Effect

send : ActorId → Message → IO Unit
send(aid, msg) = mailbox[aid] := mailbox[aid] ++ [msg]

⟦(spawn behavior)⟧ρκ = 
  let aid = fresh-actor-id()
      actor = (aid, [], behavior)
  in κ(aid) where actor-system := actor-system[aid ↦ actor]
```

### 7.4 Software Transactional Memory

```
STM τ = [TVarState → (τ × TVarState)⊥]
TVar τ = Ref τ  
TVarState = TVar ⇀ Val

atomically : STM τ → IO τ
atomically(stm) = 
  repeat
    let s₀ = current-tvar-state()
        result = stm(s₀)
    in case result of
      ⊥ → retry
      (v, s₁) → if cas-all(s₀, s₁) then return(v) else retry
```

---

## FFI Semantics

### 8.1 Foreign Function Interface

```
ForeignFunction = CSignature × NativeCode
CSignature = CType* × CType  
CType = CInt | CFloat | CString | CPtr CType | CStruct (Field*)

call-foreign : ForeignFunction → Val* → IO Val
call-foreign((arg-types, ret-type, code), args) =
  let c-args = [marshall(argi, arg-typei) | (argi, arg-typei) ∈ zip(args, arg-types)]
      c-result = invoke-native(code, c-args)  
  in return(unmarshall(c-result, ret-type))
```

### 8.2 Memory Management

```
ForeignPtr = Ptr × Finalizer
Finalizer = [] → IO Unit

with-foreign-ptr : (Ptr → IO τ) → IO τ  
with-foreign-ptr(f) = 
  ptr ← malloc()
  try f(ptr) 
  finally free(ptr)
```

---

## Metaprogramming

### 9.1 Compile-Time Evaluation

```
CompileTime τ = ReaderT CompileEnv (StateT CompileState (Either CompileError)) τ

CompileEnv = ModuleEnv × MacroEnv × TypeEnv
CompileState = SymbolTable × CodeGen

eval-at-compile-time : Expr → CompileTime Val
expand-macro : MacroCall → CompileTime Expr
```

### 9.2 Hygienic Macros

```
Syntax = Expr × SyntaxContext
SyntaxContext = Scope* × Mark*  
Scope = Binding*
Mark = ℕ

syntax-e : Syntax → Expr
datum->syntax : Expr × SyntaxContext → Syntax

expand : Syntax → CompileTime Syntax  
expand(stx) = 
  case syntax-e(stx) of
    macro-call → expand-macro-call(stx)
    (f arg ...) → expand-application(stx)
    x → expand-identifier(stx)
    literal → return(stx)
```

### 9.3 Reflection

```
reflect-type : Val → Type
reflect-environment : [] → Environment  
reflect-procedure : Procedure → ProcedureInfo

ProcedureInfo = Arity × SourceLocation × Documentation
```

---

## Operational Correspondences

### 10.1 Denotational-Operational Correspondence

For any expression e and environment ρ:

```
⟦e⟧ρ halt = v  iff  ρ ⊢ e ⇓ v   (operational semantics)
```

### 10.2 Type Soundness  

**Progress**: If `∅ ⊢ e : τ` then either `e` is a value or there exists `e'` such that `e → e'`.

**Preservation**: If `Γ ⊢ e : τ` and `e → e'` then `Γ ⊢ e' : τ`.

### 10.3 Effect Soundness

**Effect Safety**: If `Γ ⊢ e : τ ! ε` and `e ⇓ v` then all effects in the evaluation of `e` are contained in `ε`.

---

## Implementation Notes

### 11.1 Primitive Operations

The 42 core primitives are given direct denotational definitions:

```
⟦%cons⟧ = λ(v₁, v₂). (v₁, v₂)
⟦%car⟧ = λ(v). case v of (v₁, v₂) → v₁ | _ → Wrong
⟦%cdr⟧ = λ(v). case v of (v₁, v₂) → v₂ | _ → Wrong
⟦%+⟧ = λ(v₁, v₂). if v₁, v₂ ∈ Num then v₁ + v₂ else Wrong
⟦%apply⟧ = λ(f, args). f(args)
...
```

### 11.2 Garbage Collection

The semantics abstract over memory management, but the implementation uses:

```
GC : Store → Store  
GC(σ) = σ|reachable(σ)   (restrict store to reachable locations)

reachable : Store → ℘(Loc)
reachable(σ) = lfp(λS. roots ∪ {l' | l ∈ S, σ(l) contains reference to l'})
```

### 11.3 Tail Call Optimization

Tail calls are handled by the continuation-passing style:

```
⟦(f e₁ ... eₙ)⟧ρκ = ... f(v₁,...,vₙ)κ   (tail call - same continuation)
⟦(let ((x e₁)) (f e₂))⟧ρκ = ... κ'(...κ) (non-tail - new continuation)
```

---

## Conclusion

This formal semantics provides a mathematical foundation for understanding Lambdust's behavior across all language features. The denotational approach ensures compositional reasoning while the operational correspondences guarantee implementation correctness.

The semantics serve as:
1. **Specification** for language implementers  
2. **Reference** for program reasoning
3. **Foundation** for formal verification
4. **Documentation** for advanced users

For implementation details, see the Lambdust source code. For examples and tutorials, see the `examples/` directory.

---

## References

1. Kelsey, R., Clinger, W., Rees, J. (1998). "Revised⁵ Report on the Algorithmic Language Scheme"
2. Shinn, A., Cowan, J., Gleckler, A., et al. (2013). "Revised⁷ Report on the Algorithmic Language Scheme"  
3. Scheme Requests for Implementation (SRFI) Documents: https://srfi.schemers.org/
4. Stoy, J. (1977). "Denotational Semantics: The Scott-Strachey Approach"
5. Reynolds, J. (1998). "Theories of Programming Languages"
6. Pierce, B. (2002). "Types and Programming Languages"
7. Wadler, P., Findler, R.B. (2009). "Well-Typed Programs Can't Be Blamed"

---

**Maintenance Notes**: This document should be updated whenever:
- New language features are added
- SRFI implementations are modified  
- Type system extensions are made
- Semantic bugs are discovered and fixed

**Contributors**: Lambdust Development Team  
**License**: Same as Lambdust (MIT OR Apache-2.0)