# Combinator Theory Integration Design

## 🎯 Objective

Integrate combinatory logic with R7RS formal semantics to create a mathematically rigorous foundation for lambda abstraction handling, reduction systems, and theorem proving support.

## 🔬 Theoretical Foundation

### Lambda Calculus ↔ Combinatory Logic Equivalence

The key insight is that any lambda expression can be systematically translated to combinatory logic using SKI combinators:

```
S = λxyz. xz(yz)  (Substitution)
K = λxy. x        (Constant)
I = λx. x         (Identity)
```

### Translation Algorithm (Bracket Abstraction)

```
[x] x = I
[x] E = K E                    (if x not free in E)
[x] (E F) = S ([x] E) ([x] F)  (if x free in both E and F)
```

## 🏗️ Architecture Integration

### Current SemanticEvaluator Enhancement

```rust
// Existing reduction system
pub fn reduce_expression_pure(&self, expr: Expr) -> Result<Expr>

// New combinator-based reduction system
pub fn reduce_expression_combinatory(&self, expr: Expr) -> Result<Expr>
pub fn lambda_to_combinators(&self, expr: Expr) -> Result<CombinatorExpr>
pub fn combinators_to_lambda(&self, comb: CombinatorExpr) -> Result<Expr>
```

### Combinator Expression Type

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum CombinatorExpr {
    // Basic combinators
    S,
    K,
    I,
    
    // Application
    App(Box<CombinatorExpr>, Box<CombinatorExpr>),
    
    // Atomic values (literals, variables)
    Atomic(Expr),
    
    // Composite combinators for efficiency
    B,  // B = S(KS)K = λxyz. x(yz)  (Composition)
    C,  // C = S(S(K(S(KS)K))S)(KK) = λxyz. xzy  (Flip)
    W,  // W = SS(SK) = λxy. xyy  (Duplication)
}
```

## 🎨 Implementation Strategy

### Phase 1: Basic Combinator System
1. **CombinatorExpr type definition**
2. **SKI reduction rules implementation**
3. **Lambda-to-combinator translation**
4. **Combinator-to-lambda back-translation**

### Phase 2: SemanticEvaluator Integration
1. **Combinator reduction in reduce_expression_pure**
2. **Lambda abstraction handling via combinators**
3. **Verification against pure lambda semantics**

### Phase 3: Theorem Proving Support
1. **Combinator reduction proofs**
2. **Equivalence proofs between lambda and combinator forms**
3. **Agda/Coq integration preparation**

## 🔧 Technical Benefits

### 1. Mathematical Rigor
- **Formal reduction system**: Combinator reductions are well-defined and terminating
- **Proof-friendly**: Easier to prove properties in combinatory logic
- **Canonical forms**: Unique normal forms for expressions

### 2. Optimization Opportunities
- **Efficient representations**: Some patterns more efficient as combinators
- **Reduction strategies**: Different reduction orders for performance
- **Compilation target**: Combinators as intermediate representation

### 3. Correctness Guarantees
- **Semantic preservation**: Translation preserves R7RS semantics
- **Bidirectional translation**: Round-trip correctness guarantees
- **Formal verification**: Machine-checkable proofs of correctness

## 📚 Reduction Rules Implementation

### Basic SKI Rules
```rust
impl CombinatorExpr {
    pub fn reduce_step(&self) -> Option<CombinatorExpr> {
        match self {
            // S x y z → x z (y z)
            App(App(App(S, x), y), z) => {
                Some(App(
                    App(x.clone(), z.clone()),
                    App(y.clone(), z.clone())
                ))
            }
            
            // K x y → x
            App(App(K, x), _y) => Some(x.as_ref().clone()),
            
            // I x → x
            App(I, x) => Some(x.as_ref().clone()),
            
            // Recurse into applications
            App(f, x) => {
                if let Some(f_reduced) = f.reduce_step() {
                    Some(App(Box::new(f_reduced), x.clone()))
                } else if let Some(x_reduced) = x.reduce_step() {
                    Some(App(f.clone(), Box::new(x_reduced)))
                } else {
                    None
                }
            }
            
            _ => None
        }
    }
}
```

### Extended Combinator Set
```rust
// B x y z → x (y z)  (Composition)
App(App(App(B, x), y), z) => {
    Some(App(x.clone(), App(y.clone(), z.clone())))
}

// C x y z → x z y  (Flip)
App(App(App(C, x), y), z) => {
    Some(App(App(x.clone(), z.clone()), y.clone()))
}

// W x y → x y y  (Duplication)
App(App(W, x), y) => {
    Some(App(App(x.clone(), y.clone()), y.clone()))
}
```

## 🧪 Testing Strategy

### Correctness Tests
```rust
#[test]
fn test_lambda_combinator_equivalence() {
    let lambda_expr = parse("(lambda (x) (lambda (y) (x y)))").unwrap();
    let combinator_expr = lambda_to_combinators(lambda_expr.clone()).unwrap();
    let back_to_lambda = combinators_to_lambda(combinator_expr).unwrap();
    
    assert_semantically_equivalent(lambda_expr, back_to_lambda);
}
```

### Reduction Tests
```rust
#[test]
fn test_ski_reduction() {
    // S K K x → x  (Church encoding of I)
    let expr = App(App(App(S, K), K), Atomic(Variable("x")));
    let reduced = expr.reduce_to_normal_form().unwrap();
    assert_eq!(reduced, Atomic(Variable("x")));
}
```

## 🔗 Integration with Existing Systems

### SemanticEvaluator Enhancement
```rust
impl SemanticEvaluator {
    /// Reduce expression using combinatory logic
    pub fn reduce_expression_combinatory(&self, expr: Expr) -> Result<Expr> {
        // 1. Convert lambda abstractions to combinators
        let combinators = self.lambda_to_combinators(expr)?;
        
        // 2. Apply combinator reductions
        let reduced_combinators = combinators.reduce_to_normal_form()?;
        
        // 3. Convert back to lambda form
        let reduced_expr = self.combinators_to_lambda(reduced_combinators)?;
        
        // 4. Apply standard S-expression reductions
        self.reduce_expression_pure(reduced_expr)
    }
}
```

### RuntimeExecutor Integration
```rust
impl RuntimeExecutor {
    fn eval_with_combinator_optimizations(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Use combinator reductions for lambda-heavy expressions
        let reduced_expr = self.semantic_evaluator
            .reduce_expression_combinatory(expr)?;
        
        self.semantic_evaluator.eval_pure(reduced_expr, env, cont)
    }
}
```

## 📖 Mathematical Properties

### Church-Rosser Property
- **Confluence**: Different reduction orders yield the same result
- **Termination**: All reductions terminate (strongly normalizing)
- **Uniqueness**: Normal forms are unique

### Semantic Preservation
- **R7RS compliance**: All reductions preserve R7RS semantics
- **Extensional equality**: λx. f x ≡ f (when x not free in f)
- **Reduction correctness**: Each step preserves meaning

## 🎯 Future Directions

### Agda/Coq Integration
```agda
-- Combinator reduction correctness proof
combinator-reduction-correct : ∀ {A} (e : CombinatorExpr A) →
  semantics (reduce-step e) ≡ semantics e
```

### Advanced Optimizations
- **Supercombinators**: Specialized combinators for common patterns
- **Optimal reduction**: Lamping's optimal reduction algorithm
- **Parallel reduction**: Concurrent combinator evaluation

## 💡 Implementation Priority

1. **High Priority**: Basic SKI system with lambda translation
2. **Medium Priority**: Extended combinator set (BCYW)
3. **Low Priority**: Advanced optimizations and parallel reduction

This design provides a solid foundation for integrating combinatory logic with the existing R7RS evaluator while maintaining mathematical rigor and correctness guarantees.