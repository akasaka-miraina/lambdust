//! Operational semantics domain model for Lambdust's monadic evaluator.
//!
//! This module implements the mathematical foundation of evaluation contexts
//! and continuation capture according to R7RS operational semantics.
//!
//! The key insight is that evaluation contexts represent "the computation
//! surrounding a redex" and continuations capture "the rest of the computation".

use crate::ast::{Expr, Spanned};
use crate::diagnostics::{Result, Span};
use crate::eval::{Value, Environment};
use std::collections::VecDeque;
use std::rc::Rc;

/// Evaluation context - represents the "hole" where computation continues.
///
/// Mathematically, this corresponds to the evaluation context E in:
/// E ::= [] | (E e₁...eₙ) | (v₁...vᵢ E eᵢ₊₁...eₙ) | ...
///
/// The context captures the operational semantic notion of "what to do
/// with the result of the current computation".
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Stack of context frames from outermost to innermost
    /// The top of the stack is the "immediate" context
    frames: VecDeque<ContextFrame>,
    
    /// The environment where this context was captured
    captured_environment: std::sync::Arc<super::value::ThreadSafeEnvironment>,
    
    /// Unique identifier for debugging and matching
    context_id: ContextId,
    
    /// Span information for error reporting
    span: Option<Span>,
}

/// A single frame in the evaluation context.
///
/// Each frame represents one "layer" of the context - one place where
/// we're waiting for a sub-computation to complete.
#[derive(Debug, Clone)]
pub enum ContextFrame {
    /// Application context: ([] e₁ e₂ ... eₙ)
    /// We're waiting for the operator to be evaluated
    ApplicationOperator {
        operands: Vec<Spanned<Expr>>,
        environment: Rc<Environment>,
        span: Span,
    },
    
    /// Application context: (proc v₁ ... vᵢ [] eᵢ₊₁ ... eₙ)
    /// We're waiting for argument i to be evaluated
    ApplicationOperand {
        procedure: Value,
        evaluated_args: Vec<Value>,
        pending_args: Vec<Spanned<Expr>>,
        environment: Rc<Environment>,
        span: Span,
    },
    
    /// Conditional context: (if [] then-branch else-branch)
    Conditional {
        then_branch: Spanned<Expr>,
        else_branch: Option<Spanned<Expr>>,
        environment: Rc<Environment>,
        span: Span,
    },
    
    /// Assignment context: (set! var [])
    Assignment {
        variable: String,
        environment: Rc<Environment>,
        span: Span,
    },
    
    /// Begin sequence context: (begin v₁ ... vᵢ [] eᵢ₊₁ ... eₙ)
    Sequence {
        evaluated_exprs: Vec<Value>,
        pending_exprs: Vec<Spanned<Expr>>,
        environment: Rc<Environment>,
        span: Span,
    },
    
    /// Lambda body context - for proper tail call semantics
    LambdaBody {
        procedure_name: Option<String>,
        environment: Rc<Environment>,
        span: Span,
    },
    
    /// Let binding context: (let ((var₁ val₁) ... (varᵢ []) ... (varₙ valₙ)) body)
    LetBinding {
        bound_vars: Vec<(String, Value)>,
        current_var: String,
        pending_bindings: Vec<(String, Spanned<Expr>)>,
        body: Vec<Spanned<Expr>>,
        environment: Rc<Environment>,
        span: Span,
    },
    
    /// Call/cc context - special handling for continuation capture
    CallCC {
        /// The procedure that will receive the continuation
        procedure: Value,
        environment: Rc<Environment>,
        span: Span,
    },
}

/// Unique identifier for evaluation contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextId(u64);

/// Generator for unique context IDs
static CONTEXT_ID_COUNTER: std::sync::atomic::AtomicU64 = 
    std::sync::atomic::AtomicU64::new(1);

fn next_context_id() -> ContextId {
    ContextId(CONTEXT_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
}

/// Redex - a reducible expression.
///
/// In operational semantics, the computation state is (E, e) where:
/// - E is the evaluation context
/// - e is the redex (the expression currently being reduced)
#[derive(Debug, Clone)]
pub struct Redex {
    /// The expression being evaluated
    pub expression: Spanned<Expr>,
    
    /// The environment for evaluation
    pub environment: Rc<Environment>,
    
    /// Additional metadata
    pub metadata: RedexMetadata,
}

/// Metadata associated with a redex
#[derive(Debug, Clone)]
pub struct RedexMetadata {
    /// Whether this is a tail position
    pub is_tail_position: bool,
    
    /// Stack depth for debugging
    pub stack_depth: usize,
    
    /// Generation for GC
    pub generation: u64,
}

/// Computational state in the abstract machine.
///
/// This represents the complete state: (E, e) where E is context and e is redex
#[derive(Debug, Clone)]
pub struct ComputationState {
    /// Current evaluation context
    pub context: EvaluationContext,
    
    /// Current redex being evaluated
    pub redex: Redex,
    
    /// Additional machine state
    pub machine_state: MachineState,
}

/// Additional state maintained by the abstract machine
#[derive(Debug, Clone)]
pub struct MachineState {
    /// Current generation for environments
    pub generation: u64,
    
    /// Tail call optimization flag
    pub in_tail_position: bool,
    
    /// Stack depth counter
    pub stack_depth: usize,
}

impl EvaluationContext {
    /// Create a new empty evaluation context (top-level)
    pub fn empty(environment: Rc<Environment>) -> Self {
        Self {
            frames: VecDeque::new(),
            captured_environment: super::value::ThreadSafeEnvironment::from_legacy(&environment),
            context_id: next_context_id(),
            span: None,
        }
    }
    
    /// Create a context with a single frame
    pub fn single_frame(frame: ContextFrame, environment: Rc<Environment>) -> Self {
        let mut frames = VecDeque::new();
        frames.push_back(frame);
        
        Self {
            frames,
            captured_environment: super::value::ThreadSafeEnvironment::from_legacy(&environment),
            context_id: next_context_id(),
            span: None,
        }
    }
    
    /// Get the captured environment
    pub fn environment(&self) -> &std::sync::Arc<super::value::ThreadSafeEnvironment> {
        &self.captured_environment
    }
    
    /// Get the captured environment as legacy Rc<Environment> (for compatibility)
    pub fn environment_legacy(&self) -> Rc<Environment> {
        self.captured_environment.to_legacy()
    }
    
    /// Push a new frame onto the context stack
    pub fn push_frame(&mut self, frame: ContextFrame) {
        self.frames.push_back(frame);
    }
    
    /// Pop the top frame from the context stack
    pub fn pop_frame(&mut self) -> Option<ContextFrame> {
        self.frames.pop_back()
    }
    
    /// Get the top frame without removing it
    pub fn peek_frame(&self) -> Option<&ContextFrame> {
        self.frames.back()
    }
    
    /// Check if this context is empty (top-level)
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
    
    /// Get the depth of this context
    pub fn depth(&self) -> usize {
        self.frames.len()
    }
    
    /// Compose this context with another (this becomes outer context)
    pub fn compose(mut self, inner: EvaluationContext) -> EvaluationContext {
        // The mathematical composition: if we have contexts E₁ and E₂,
        // then E₁[E₂[•]] is the composition
        for frame in inner.frames {
            self.frames.push_back(frame);
        }
        self
    }
    
    /// Extract the continuation represented by this context.
    ///
    /// This is the key operation for call/cc - it "reifies" the evaluation
    /// context as a Scheme continuation value.
    pub fn to_continuation(&self) -> crate::eval::value::Continuation {
        use crate::eval::value::Continuation;
        use std::sync::Arc;
        
        // Convert context frames to continuation stack
        let stack = self.frames.iter()
            .map(|frame| self.context_frame_to_stack_frame(frame))
            .collect();
        
        Continuation::new(
            stack,
            self.captured_environment.clone()),
            self.context_id.0,
            None, // current_expr - would be set in full implementation
        )
    }
    
    /// Apply this context to a value ("fill the hole")
    ///
    /// This implements the operational semantic rule:
    /// If we have context E and value v, then E[v] is the result
    pub fn apply_to_value(&self, value: Value) -> Result<ComputationState> {
        if self.is_empty() {
            // Empty context - value is the final result
            // This shouldn't happen in normal evaluation, but we handle it gracefully
            return Ok(ComputationState {
                context: EvaluationContext::empty(self.environment_legacy()),
                redex: Redex {
                    expression: Spanned {
                        inner: Expr::Quote(Box::new(Spanned {
                            inner: value.to_expr()?,
                            span: Span::default(),
                        })),
                        span: Span::default(),
                    },
                    environment: self.environment_legacy(),
                    metadata: RedexMetadata {
                        is_tail_position: true,
                        stack_depth: 0,
                        generation: 0,
                    },
                },
                machine_state: MachineState {
                    generation: 0,
                    in_tail_position: true,
                    stack_depth: 0,
                },
            });
        }
        
        let mut new_context = self.clone());
        let top_frame = new_context.pop_frame().unwrap();
        
        // Create new redex based on the top frame and the value
        let (new_expr, new_env) = self.fill_frame_with_value(&top_frame, value)?;
        
        Ok(ComputationState {
            context: new_context,
            redex: Redex {
                expression: new_expr,
                environment: new_env,
                metadata: RedexMetadata {
                    is_tail_position: self.frames.len() == 1, // tail if only one frame left
                    stack_depth: self.frames.len(),
                    generation: 0, // Would be set by the evaluator
                },
            },
            machine_state: MachineState {
                generation: 0,
                in_tail_position: self.frames.len() == 1,
                stack_depth: self.frames.len(),
            },
        })
    }
    
    /// Convert a context frame to a stack frame (for continuation representation)
    fn context_frame_to_stack_frame(&self, frame: &ContextFrame) -> crate::eval::value::Frame {
        // TODO: Implement proper conversion from ContextFrame to value::Frame
        // This requires understanding the Frame enum structure and creating proper constructors
        use crate::eval::value::Frame;
        use std::sync::Arc;
        
        // For now, return a simple CallCC frame to get compilation working
        Frame::CallCC {
            environment: Arc::new(crate::eval::value::ThreadSafeEnvironment::default()),
            source: crate::diagnostics::Span::new(0, 0),
        }
    }
    
    /// Fill a context frame with a value to create a new expression
    fn fill_frame_with_value(
        &self, 
        frame: &ContextFrame, 
        value: Value
    ) -> Result<(Spanned<Expr>, Rc<Environment>)> {
        match frame {
            ContextFrame::ApplicationOperator { operands, environment, span } => {
                // The value is the procedure, now we need to evaluate the operands
                Ok((
                    Spanned {
                        inner: Expr::Application {
                            operator: Box::new(Spanned {
                                inner: value.to_expr()?,
                                span: *span,
                            }),
                            operands: operands.clone()),
                        },
                        span: *span,
                    },
                    environment.clone()),
                ))
            }
            
            ContextFrame::ApplicationOperand { 
                procedure, 
                evaluated_args, 
                pending_args, 
                environment, 
                span 
            } => {
                // Add this value to evaluated args
                let mut new_evaluated = evaluated_args.clone());
                new_evaluated.push(value);
                
                if pending_args.is_empty() {
                    // All arguments evaluated - ready to apply
                    Ok((
                        Spanned {
                            inner: Expr::Application {
                                operator: Box::new(Spanned {
                                    inner: procedure.to_expr()?,
                                    span: *span,
                                }),
                                operands: new_evaluated.into_iter()
                                    .map(|v| Spanned {
                                        inner: v.to_expr().unwrap_or(Expr::Literal(crate::ast::Literal::Nil)),
                                        span: *span,
                                    })
                                    .collect(),
                            },
                            span: *span,
                        },
                        environment.clone()),
                    ))
                } else {
                    // Still have more arguments to evaluate
                    let next_arg = pending_args[0].clone());
                    Ok((next_arg, environment.clone()))
                }
            }
            
            ContextFrame::Conditional { then_branch, else_branch, environment, span } => {
                // Use the value as the condition
                if value.is_truthy() {
                    Ok((then_branch.clone()), environment.clone()))
                } else if let Some(else_expr) = else_branch {
                    Ok((else_expr.clone()), environment.clone()))
                } else {
                    Ok((
                        Spanned {
                            inner: Expr::Literal(crate::ast::Literal::Unspecified),
                            span: *span,
                        },
                        environment.clone()),
                    ))
                }
            }
            
            _ => {
                // Simplified handling for other frame types
                Ok((
                    Spanned {
                        inner: value.to_expr()?,
                        span: Span::default(),
                    },
                    self.environment_legacy(),
                ))
            }
        }
    }
    
    /// Get the context ID for debugging and matching
    pub fn id(&self) -> ContextId {
        self.context_id
    }
    
    // Environment method is defined earlier with ThreadSafeEnvironment return type
}

impl ComputationState {
    /// Create a new computation state
    pub fn new(context: EvaluationContext, redex: Redex) -> Self {
        let machine_state = MachineState {
            generation: redex.metadata.generation,
            in_tail_position: redex.metadata.is_tail_position,
            stack_depth: redex.metadata.stack_depth,
        };
        
        Self {
            context,
            redex,
            machine_state,
        }
    }
    
    /// Check if this computation is in a tail position
    pub fn is_tail_position(&self) -> bool {
        self.machine_state.in_tail_position
    }
    
    /// Get the current stack depth
    pub fn stack_depth(&self) -> usize {
        self.machine_state.stack_depth
    }
}

/// Extension trait to convert Values back to Expressions (for context filling)
trait ValueToExpr {
    fn to_expr(&self) -> Result<Expr>;
}

impl ValueToExpr for Value {
    fn to_expr(&self) -> Result<Expr> {
        match self {
            Value::Literal(lit) => Ok(Expr::Literal(lit.clone())),
            Value::Symbol(sym) => Ok(Expr::Identifier(format!("symbol_{}", sym.id()))),
            Value::Pair(car, cdr) => {
                // Convert pair to application or list as appropriate
                Ok(Expr::Literal(crate::ast::Literal::Nil)) // Simplified
            }
            _ => {
                // For complex values, we use a quote
                Ok(Expr::Quote(Box::new(Spanned {
                    inner: Expr::Literal(crate::ast::Literal::Nil), // Simplified
                    span: Span::default(),
                })))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_context() {
        let env = Rc::new(Environment::new(None, 0));
        let ctx = EvaluationContext::empty(env.clone());
        
        assert!(ctx.is_empty());
        assert_eq!(ctx.depth(), 0);
        assert!(ctx.peek_frame().is_none());
    }
    
    #[test]
    fn test_context_composition() {
        let env = Rc::new(Environment::new(None, 0));
        let frame1 = ContextFrame::Conditional {
            then_branch: Spanned {
                inner: Expr::Literal(crate::ast::Literal::Number(42.0)),
                span: Span::default(),
            },
            else_branch: None,
            environment: env.clone()),
            span: Span::default(),
        };
        
        let ctx1 = EvaluationContext::single_frame(frame1, env.clone());
        let ctx2 = EvaluationContext::empty(env.clone());
        
        let composed = ctx2.compose(ctx1);
        assert_eq!(composed.depth(), 1);
    }
    
    #[test]
    fn test_context_id_uniqueness() {
        let env = Rc::new(Environment::new(None, 0));
        let ctx1 = EvaluationContext::empty(env.clone());
        let ctx2 = EvaluationContext::empty(env.clone());
        
        assert_ne!(ctx1.id(), ctx2.id());
    }
}
