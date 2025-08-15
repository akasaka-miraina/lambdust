# Lambdust Formal Denotational Semantics

## Overview

This document provides a formal denotational semantics for Lambdust, extending R7RS-small with advanced language features including gradual dependent typing, effect systems, concurrency, and FFI. The semantics is designed to support mechanical verification with Isabelle/HOL and automated translation from Event-B specifications.

The notation extends the R7RS conventions with Lambdust-specific domains and operations. The core R7RS semantics is preserved as a conservative extension.

**Notation Summary:**
```
⟨ … ⟩       sequence formation
s ↓ k       kth member of the sequence s (1-based)
#s          length of sequence s
s ⁀ t       concatenation of sequences s and t
s † k       drop the first k members of sequence s
t → a, b    McCarthy conditional "if t then a else b"
ρ[x/i]      substitution "ρ with x for i"
x in D      injection of x into domain D
x | D       projection of x to domain D
⟦P⟧         denotational meaning of program P
⊨           semantic entailment
```

The Lambdust semantics maintains R7RS compatibility while providing rigorous foundations for:
- Gradual dependent typing with type-level computation
- Effect systems with algebraic effects and handlers
- Actor-based concurrency with fault tolerance
- Memory-safe FFI with capability-based security
- Proof-carrying code via Curry-Howard correspondence

## Extended Abstract Syntax

**Domain Classifications:**
```
K ∈ Con     constants, including quotations
I ∈ Ide     identifiers (variables)
T ∈ Typ     type expressions
F ∈ Eff     effect specifications
E ∈ Exp     expressions
C ∈ Com = Exp  commands
P ∈ Prf     proof terms
```

**Extended Grammar:**

### R7RS Core (unchanged)
```scheme
Exp → K | I | (E₀ E*)
    | (lambda (I*) C* E₀)
    | (lambda (I* . I) C* E₀)
    | (lambda I C* E₀)
    | (if E₀ E₁ E₂) | (if E₀ E₁)
    | (set! I E)
```

### Lambdust Type Extensions
```scheme
    | (: I T)                      ; type annotation
    | (Pi (I T₁) T₂)              ; dependent function type
    | (Sigma (I T₁) T₂)           ; dependent product type
    | (Refinement (I T) E)        ; refinement type
    | (the T E)                   ; explicit type ascription
```

### Effect System
```scheme
    | (perform I E)               ; effect invocation
    | (handle E (I (I* I* E)*))   ; effect handler
    | (lift E)                    ; effect lifting
```

### Concurrency
```scheme
    | (spawn E)                   ; actor spawning
    | (send E E)                  ; message sending
    | (receive ((E E)*))          ; pattern-based message reception
    | (future E)                  ; asynchronous computation
    | (await E)                   ; synchronization
```

### FFI
```scheme
    | (foreign-call I T E*)       ; foreign function call
    | (foreign-data I T)          ; foreign data access
    | (capability I E)            ; capability-based access
```

### Proof System
```scheme
    | (prove T P)                 ; proof term construction
    | (extract P)                 ; program extraction from proof
    | (qed P)                     ; proof completion
```

## Extended Domain Equations

### R7RS Domains (preserved)
```
α ∈ L                               locations
ν ∈ ℕ                               natural numbers
    T = {false, true}               booleans
    Q                               symbols
    H                               characters
    R                               numbers
    Eₚ = L × L × T                  pairs
    Eᵥ = L* × T                     vectors
    Eₛ = L* × T                     strings
    M = {false, true, null,         miscellaneous
         undefined, unspecified}
φ ∈ F = L × (E* → P → K → C)       procedure values
ε ∈ E = Q + H + R + Eₚ + Eᵥ +       expressed values
        Eₛ + M + F + …
σ ∈ S = L → (E × T)                stores
ρ ∈ U = Ide → L                    environments
θ ∈ C = S → A                      command continuations
κ ∈ K = E* → C                     expression continuations
    A                               answers
    X                               errors
ω ∈ P = (F × F × P) + {root}       dynamic points
```

### Lambdust Type System
```
τ ∈ Typ = Typ_base + Typ_dep + Typ_eff                types
    Typ_base = {Dynamic, Number, String,              base types
                Boolean, Symbol}
    Typ_dep = Π-types + Σ-types + Refinements        dependent types
    Typ_eff = Typ → Eff → Typ                        effectful types
δ ∈ Typ_ctx = Ide → Typ                              type environments
ξ ∈ Typ_sub = Typ × Typ                              subtyping relation
```

### Effect System
```
ψ ∈ Eff = 𝒫(EffectLabel)                            effect sets
    EffectLabel = Q × Typ × Typ                     labeled effects
η ∈ Handler = EffectLabel → F                       effect handlers
μ ∈ Monad = Type → Type                             monadic types
```

### Concurrency System
```
α ∈ ActorId = ℕ                                     actor identifiers
λ ∈ Message = E × ActorId                           messages
χ ∈ Mailbox = Message*                              actor mailboxes
ζ ∈ ActorSystem = ActorId → Actor                   actor system state
    Actor = Behavior × χ × State                    actor structure
φf ∈ Future = E + {pending}                         future values
```

### FFI System
```
γ ∈ CType = {int32, int64, float64, ptr, ...}       C data types
ι ∈ FFIBinding = Q × CType × L                      foreign bindings
κc ∈ Capability = Resource × Permission              access capabilities
    Resource = LibraryId × FunctionId               foreign resources
    Permission = {read, write, exec}                access permissions
```

### Proof System
```
π ∈ Prf = ProofTerm                                 proof terms
    ProofTerm = λ-terms + Induction + Tactics       proof structures
φ ∈ Proposition = Typ                               propositions as types
Γ ∈ Context = Ide → Typ                            proof contexts
```

## Extended Semantic Functions

### R7RS Core (preserved)
```
𝒦 : Con → E
ℰ : Exp → U → P → K → C
ℰ* : Exp* → U → P → K → C
𝒞 : Com* → U → P → C → C
```

### Lambdust Extensions
```
𝒯 : Typ → TypeDenotation
ℱ : Eff → EffectDenotation
𝒜 : ActorExp → ActorSystem → ActorSystem
ℐ : FFIExp → Capability → E
𝒫 : Prf → ProofDenotation
```

## Type System Semantics

The type system provides gradual dependent typing with four levels:
1. **Dynamic**: No static checking, full R7RS compatibility
2. **Contracts**: Runtime type checking with gradual migration  
3. **Static**: Hindley-Milner style inference with extensions
4. **Dependent**: Full dependent types with proof obligations

### Dynamic Types
```
𝒯⟦Dynamic⟧ = λρ ε . ε
```

### Dependent Function Types (Π-types)
```
𝒯⟦(Pi (I T₁) T₂)⟧ = 
  λρ . {ε | ε ∈ F ∧
       ∀ε' ∈ 𝒯⟦T₁⟧ρ .
       applicate ε ⟨ε'⟩ ∈ 𝒯⟦T₂⟧(ρ[ε'/I])}
```

### Refinement Types  
```
𝒯⟦(Refinement (I T) E)⟧ =
  λρ . {ε | ε ∈ 𝒯⟦T⟧ρ ∧
       truish(ℰ⟦E⟧(ρ[ε/I]))}
```

## Effect System Semantics

Effects are modeled as algebraic effects with handlers. The effect system ensures that side effects are properly tracked and contained.

### Effect Invocation
```
ℰ⟦(perform I E)⟧ =
  λρ ω κ . ℰ⟦E⟧ ρ ω
    (single(λε .
      invoke-effect I ε ω κ))
```

### Effect Handling
```
ℰ⟦(handle E (I (Iᵢ I*ᵢ Eᵢ)*))⟧ =
  λρ ω κ .
    ℰ⟦E⟧ ρ
      (extend-handler ω (make-handler (Iᵢ I*ᵢ Eᵢ)* ρ))
      κ
```

### Auxiliary Effect Functions
```
invoke-effect : Q → E → P → K → C
invoke-effect =
  λlabel ε ω κ .
    find-handler label ω (
      λhandler . handler ε κ,
      wrong("unhandled effect: " ++ label))
```

## Actor System Semantics

The actor system provides fault-tolerant concurrency with supervision trees and location transparency.

### Actor Spawning
```
ℰ⟦(spawn E)⟧ =
  λρ ω κ . λζ .
    fresh-actor-id ζ (
      λα .
        send α κ
          (spawn-actor α (ℰ⟦E⟧ ρ ω) ζ))
```

### Message Sending
```
ℰ⟦(send E₁ E₂)⟧ =
  λρ ω κ .
    ℰ⟦E₁⟧ ρ ω (
      single(λα .
        ℰ⟦E₂⟧ ρ ω (
          single(λε .
            enqueue-message α ε κ))))
```

### Auxiliary Actor Functions
```
spawn-actor : ActorId → E → ActorSystem → ActorSystem
spawn-actor =
  λα behavior ζ .
    ζ[α ↦ (behavior, ⟨⟩, fresh-state)]
```

## FFI System Semantics

The FFI provides capability-based access to foreign functions with automatic memory management and type safety.

### Foreign Function Calls
```
ℰ⟦(foreign-call I T E*)⟧ =
  λρ ω κ .
    ℰ*⟦E*⟧ ρ ω (
      λε* .
        check-capability I (
          λcap .
            marshal-call I T ε* cap κ))
```

### Foreign Call Marshalling
```
marshal-call : Q → Typ → E* → Capability → K → C
marshal-call =
  λfname ret-type ε* cap κ .
    convert-args ε* (
      λc-args .
        invoke-foreign fname c-args (
          λc-result .
            convert-result ret-type c-result κ))
```

## Proof System Semantics

The proof system implements Curry-Howard correspondence, enabling program extraction from constructive proofs.

### Proof Construction
```
𝒫⟦(prove T P)⟧ =
  λΓ .
    check-proof P T Γ (
      λverified .
        verified → 𝒫⟦P⟧Γ,
        wrong("proof verification failed"))
```

### Program Extraction
```
𝒫⟦(extract P)⟧ =
  λΓ .
    extract-program (𝒫⟦P⟧Γ)
```

### Proof Verification
```
check-proof : Prf → Typ → Context → (T → C) → C
check-proof =
  λproof prop Γ κ .
    type-of-proof proof Γ (
      λinferred .
        types-equivalent inferred prop →
          κ true,
          κ false)
```

## Conservative Extension Properties

The Lambdust semantics satisfies the following key properties:

### R7RS Compatibility
For any R7RS-compliant program P:
```
ℰ_Lambdust⟦P⟧ = ℰ_R7RS⟦P⟧
```
when run in Dynamic mode.

### Type Safety
Well-typed programs cannot go wrong:
```
⊢ P : τ  ⟹  ∀σ . ℰ⟦P⟧ ∅ root (λε . ε) ≠ wrong(·)
```

### Effect Safety  
Effects are properly contained by handlers:
```
⊢ P : τ in Ψ  ⟹  effects(ℰ⟦P⟧) ⊆ Ψ
```

### Memory Safety
FFI operations cannot access unauthorized memory:
```
safe(capability) ⟹ safe(ℐ⟦ffi-call⟧ capability)
```

### Proof Soundness
Extracted programs satisfy their specifications:
```
⊢ P : proof-of(φ) ⟹ ℰ⟦extract(P)⟧ ⊨ φ
```

## Relationship to Event-B and Isabelle/HOL

### Event-B Translation
Each Lambdust construct maps to Event-B machines with precisely defined refinement relationships:

- **Type refinement**: `Dynamic ⊑ Contracts ⊑ Static ⊑ Dependent`
- **Effect refinement**: Handlers refine raw computational effects  
- **Concurrency refinement**: Actor protocols refine message-passing specifications
- **FFI refinement**: Capability constraints refine memory access patterns

### Isabelle/HOL Verification
Domain equations and semantic functions are directly expressible in Isabelle/HOL:

```isabelle
datatype lambdust_exp = 
    Const const
  | Var ident  
  | App lambdust_exp "lambdust_exp list"
  | Lambda "ident list" "lambdust_exp list" lambdust_exp
  | TypeAnnot ident lambdust_type
  | Pi ident lambdust_type lambdust_type
  | Perform ident lambdust_exp
  | Spawn lambdust_exp
  | ForeignCall ident lambdust_type "lambdust_exp list"
  | Prove lambdust_type proof_term
```

### Proof Carrying Code
The proof system enables automatic generation of machine-checkable correctness certificates:

1. **Specification**: Event-B machines with proof obligations
2. **Implementation**: Lambdust programs with dependent types
3. **Certification**: Extracted proof terms verified in Isabelle/HOL
4. **Deployment**: Runtime with certified safety properties

## Implementation Guidelines

### Gradual Migration Strategy
```
Dynamic → Contracts → Static → Dependent
   ↓         ↓          ↓         ↓
Runtime   Runtime    Compile   Compile+Proof
checks    checks     checks    obligations
```

### Effect Type Inference
```
Γ ⊢ e : τ ! ∅           (Pure expressions)
Γ ⊢ perform op e : τ ! {op}  (Effect introduction)  
Γ ⊢ handle e h : τ ! (ψ \ handled(h))  (Effect elimination)
```

### Actor Type System
```
Γ ⊢ spawn e : ActorRef⟨τ⟩     where Γ ⊢ e : τ → τ  
Γ ⊢ send a m : Unit           where Γ ⊢ a : ActorRef⟨τ⟩, Γ ⊢ m : τ
Γ ⊢ receive p : τ             where pattern p matches messages of type τ
```

### FFI Safety Constraints
```
⊢ foreign-call f : (T₁ × … × Tₙ) → T requires capability(f, read)
⊢ foreign-data d : T requires capability(d, access)
memory-safe(op) ∧ type-safe(op) ∧ capability-authorized(op) ⟹ safe(op)
```

## Conclusion

This formal semantics provides a mathematically rigorous foundation for Lambdust, enabling:

- **Theoretical Rigor**: Precise meaning for all language constructs
- **Practical Implementation**: Clear guidance for language implementers  
- **Formal Verification**: Machine-checkable proofs of program properties
- **Tool Integration**: Seamless interoperation with formal methods tools

The conservative extension property ensures that existing Scheme code continues to work while providing a smooth migration path to advanced type system features and formal verification capabilities.

The complete development establishes Lambdust as a bridge between practical functional programming and rigorous formal methods, making high-assurance software development accessible to a broader programming community.