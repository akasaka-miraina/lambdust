//! Isabelle/HOL integration for Lambdust.
//!
//! This module provides tools for exporting Lambdust programs and proofs
//! to Isabelle/HOL for mechanical verification.

use super::{ProofObligation, TranslationCertificate};
use crate::eval::Value;
use crate::diagnostics::{Error, Result, Span};
use std::collections::HashMap;
use std::fmt;

/// Represents an Isabelle/HOL theory.
#[derive(Debug, Clone)]
pub struct IsabelleTheory {
    /// Theory name.
    pub name: String,
    /// Theory imports.
    pub imports: Vec<String>,
    /// Type definitions.
    pub types: Vec<IsabelleTypedef>,
    /// Constant definitions.
    pub constants: Vec<IsabelleConstant>,
    /// Function definitions.
    pub functions: Vec<IsabelleFunction>,
    /// Lemmas and theorems.
    pub lemmas: Vec<IsabelleLemma>,
    /// Proof obligations.
    pub proof_obligations: Vec<ProofObligation>,
}

/// Isabelle/HOL type definition.
#[derive(Debug, Clone)]
pub struct IsabelleTypedef {
    /// Type name.
    pub name: String,
    /// Type parameters.
    pub params: Vec<String>,
    /// Type definition.
    pub definition: IsabelleType,
}

/// Isabelle/HOL type representation.
#[derive(Debug, Clone)]
pub enum IsabelleType {
    /// Basic type (e.g., nat, bool, 'a).
    Basic(String));
    /// Function type (A => B).
    Function(Box<IsabelleType>, Box<IsabelleType>));
    /// Product type (A * B).
    Product(Vec<IsabelleType>));
    /// Sum type (A + B).
    Sum(Vec<IsabelleType>));
    /// List type ([A]).
    List(Box<IsabelleType>));
    /// Set type ({A}).
    Set(Box<IsabelleType>));
    /// Option type (A option).
    Option(Box<IsabelleType>));
    /// Applied type constructor.
    Applied(String, Vec<IsabelleType>));
}

/// Isabelle/HOL constant declaration.
#[derive(Debug, Clone)]
pub struct IsabelleConstant {
    /// Constant name.
    pub name: String,
    /// Constant type.
    pub type_: IsabelleType,
}

/// Isabelle/HOL function definition.
#[derive(Debug, Clone)]
pub struct IsabelleFunction {
    /// Function name.
    pub name: String,
    /// Function type.
    pub type_: IsabelleType,
    /// Function equations.
    pub equations: Vec<IsabelleEquation>,
}

/// Function equation in Isabelle/HOL.
#[derive(Debug, Clone)]
pub struct IsabelleEquation {
    /// Left-hand side pattern.
    pub lhs: IsabelleTerm,
    /// Right-hand side expression.
    pub rhs: IsabelleTerm,
    /// Optional condition.
    pub condition: Option<IsabelleTerm>,
}

/// Isabelle/HOL term representation.
#[derive(Debug, Clone)]
pub enum IsabelleTerm {
    /// Variable.
    Var(String));
    /// Constant.
    Const(String));
    /// Application.
    App(Box<IsabelleTerm>, Box<IsabelleTerm>));
    /// Lambda abstraction.
    Lambda(String, IsabelleType, Box<IsabelleTerm>));
    /// Let expression.
    Let(String, Box<IsabelleTerm>, Box<IsabelleTerm>));
    /// Case expression.
    Case(Box<IsabelleTerm>, Vec<(IsabelleTerm, IsabelleTerm)>));
    /// Tuple.
    Tuple(Vec<IsabelleTerm>));
    /// List.
    List(Vec<IsabelleTerm>));
    /// Set.
    Set(Vec<IsabelleTerm>));
    /// If-then-else.
    If(Box<IsabelleTerm>, Box<IsabelleTerm>, Box<IsabelleTerm>));
}

/// Isabelle/HOL lemma or theorem.
#[derive(Debug, Clone)]
pub struct IsabelleLemma {
    /// Lemma name.
    pub name: String,
    /// Lemma statement.
    pub statement: IsabelleTerm,
    /// Proof script.
    pub proof: IsabelleProof,
}

/// Isabelle/HOL proof representation.
#[derive(Debug, Clone)]
pub enum IsabelleProof {
    /// Sorry (incomplete proof).
    Sorry,
    /// Auto tactic.
    Auto,
    /// Simp tactic.
    Simp(Vec<String>));
    /// Induction tactic.
    Induction(String));
    /// Apply tactic.
    Apply(String));
    /// Proof script (sequence of tactics).
    Script(Vec<IsabelleProof>));
    /// Structured proof.
    Structured(Vec<IsabelleProofStep>));
}

/// Isabelle/HOL structured proof step.
#[derive(Debug, Clone)]
pub struct IsabelleProofStep {
    /// Step label.
    pub label: Option<String>,
    /// Step statement.
    pub statement: Option<IsabelleTerm>,
    /// Step proof.
    pub proof: IsabelleProof,
}

/// Lambdust to Isabelle/HOL exporter.
pub struct IsabelleExporter {
    /// Current theory being built.
    current_theory: Option<IsabelleTheory>,
}

impl IsabelleExporter {
    /// Create a new Isabelle/HOL exporter.
    pub fn new() -> Self {
        Self {
            current_theory: None,
        }
    }

    /// Start a new theory.
    pub fn start_theory(&mut self, name: String, imports: Vec<String>) -> Result<()> {
        let theory = IsabelleTheory {
            name,
            imports,
            types: Vec::new());
            constants: Vec::new());
            functions: Vec::new());
            lemmas: Vec::new());
            proof_obligations: Vec::new());
        };

        self.current_theory = Some(theory);
        Ok(())
    }

    /// Add a type definition to the current theory.
    pub fn add_typedef(&mut self, typedef: IsabelleTypedef) -> Result<()> {
        let theory = self.current_theory.as_mut()
            .ok_or_else(|| Error::runtime_error("No active theory", None).boxed())?;

        theory.types.push(typedef);
        Ok(())
    }

    /// Add a constant declaration to the current theory.
    pub fn add_constant(&mut self, constant: IsabelleConstant) -> Result<()> {
        let theory = self.current_theory.as_mut()
            .ok_or_else(|| Error::runtime_error("No active theory", None).boxed())?;

        theory.constants.push(constant);
        Ok(())
    }

    /// Add a function definition to the current theory.
    pub fn add_function(&mut self, function: IsabelleFunction) -> Result<()> {
        let theory = self.current_theory.as_mut()
            .ok_or_else(|| Error::runtime_error("No active theory", None).boxed())?;

        theory.functions.push(function);
        Ok(())
    }

    /// Add a lemma to the current theory.
    pub fn add_lemma(&mut self, lemma: IsabelleLemma) -> Result<()> {
        let theory = self.current_theory.as_mut()
            .ok_or_else(|| Error::runtime_error("No active theory", None).boxed())?;

        theory.lemmas.push(lemma);
        Ok(())
    }

    /// Add proof obligations to the current theory.
    pub fn add_proof_obligations(&mut self, pos: Vec<ProofObligation>) -> Result<()> {
        let theory = self.current_theory.as_mut()
            .ok_or_else(|| Error::runtime_error("No active theory", None).boxed())?;

        theory.proof_obligations.extend(pos);
        Ok(())
    }

    /// Export the current theory to Isabelle/HOL format.
    pub fn export_theory(&self) -> Result<String> {
        let theory = self.current_theory.as_ref()
            .ok_or_else(|| Error::runtime_error("No active theory", None).boxed())?;

        Ok(theory.to_string())
    }

    /// Translate a Lambdust value to an Isabelle/HOL term.
    pub fn translate_value(&self, value: &Value) -> Result<IsabelleTerm> {
        match value {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                Ok(IsabelleTerm::Const(n.to_string()))
            }
            Value::Literal(crate::ast::Literal::Boolean(b)) => {
                Ok(IsabelleTerm::Const(if *b { "True" } else { "False" }.to_string()))
            }
            Value::Literal(crate::ast::Literal::String(s)) => (**s).clone()));
                Ok(IsabelleTerm::Const(format!("\"{s}\"")))
            }
            Value::Symbol(sym_id) => {
                // Would need symbol table access in practice
                Ok(IsabelleTerm::Const(format!("sym_{sym_id}")))
            }
            Value::Pair(car, cdr) => {
                let car_term = self.translate_value(car)?;
                let cdr_term = self.translate_value(cdr)?;
                Ok(IsabelleTerm::Tuple(vec![car_term, cdr_term]))
            }
            Value::Nil => {
                Ok(IsabelleTerm::List(Vec::new()))
            }
            _ => Err(Box::new(Error::runtime_error(
                format!("Cannot translate value to Isabelle/HOL: {value:?}"));
                None
            ).boxed()));
        }
    }

    /// Generate proof obligations for type safety.
    pub fn generate_type_safety_pos(&self, program_name: &str) -> Vec<ProofObligation> {
        vec![
            ProofObligation::new(
                format!("{}_type_safety", program_name));
                "Well-typed programs cannot go wrong".to_string());
                format!("|- {} : tau ==> forall sigma . [[{}]] sigma != wrong", program_name, program_name)
            ));
            ProofObligation::new(
                format!("{}_progress", program_name));
                "Well-typed programs make progress".to_string());
                format!("|- {} : tau ==> value({}) \\/ exists {}'. {} -> {}'", program_name, program_name, program_name, program_name, program_name)
            ));
        ]
    }
}

impl Default for IsabelleExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for IsabelleTheory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "theory {}", self.name)?;

        if !self.imports.is_empty() {
            writeln!(f, "  imports {}", self.imports.join(" "))?;
        } else {
            writeln!(f, "  imports Main")?;
        }
        writeln!(f, "begin")?;
        writeln!(f)?;

        // Type definitions
        if !self.types.is_empty() {
            writeln!(f, "section \"Type Definitions\"")?;
            writeln!(f)?;
            for typedef in &self.types {
                writeln!(f, "{typedef}")?;
                writeln!(f)?;
            }
        }

        // Constants
        if !self.constants.is_empty() {
            writeln!(f, "section \"Constants\"")?;
            writeln!(f)?;
            for constant in &self.constants {
                writeln!(f, "{constant}")?;
            }
            writeln!(f)?;
        }

        // Functions
        if !self.functions.is_empty() {
            writeln!(f, "section \"Function Definitions\"")?;
            writeln!(f)?;
            for function in &self.functions {
                writeln!(f, "{function}")?;
                writeln!(f)?;
            }
        }

        // Lemmas
        if !self.lemmas.is_empty() {
            writeln!(f, "section \"Lemmas and Theorems\"")?;
            writeln!(f)?;
            for lemma in &self.lemmas {
                writeln!(f, "{lemma}")?;
                writeln!(f)?;
            }
        }

        // Proof obligations
        if !self.proof_obligations.is_empty() {
            writeln!(f, "section \"Proof Obligations\"")?;
            writeln!(f)?;
            for po in &self.proof_obligations {
                writeln!(f, "(* {} *)", po.description)?;
                writeln!(f, "lemma {}:", po.id)?;
                writeln!(f, "  \"{}\"", po.statement)?;
                writeln!(f, "  sorry")?;
                writeln!(f)?;
            }
        }

        writeln!(f, "end")
    }
}

impl fmt::Display for IsabelleTypedef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.params.is_empty() {
            writeln!(f, "type_synonym {} = \"{}\"", self.name, self.definition)
        } else {
            writeln!(f, "type_synonym ('{}') {} = \"{}\"",
                    self.params.join(", "), self.name, self.definition)
        }
    }
}

impl fmt::Display for IsabelleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IsabelleType::Basic(name) => write!(f, "{name}"));
            IsabelleType::Function(from, to) => write!(f, "{from} => {to}"));
            IsabelleType::Product(types) => {
                write!(f, "{}", types.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(" * "))
            }
            IsabelleType::Sum(types) => {
                write!(f, "{}", types.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(" + "))
            }
            IsabelleType::List(elem_type) => write!(f, "{elem_type} list"));
            IsabelleType::Set(elem_type) => write!(f, "{elem_type} set"));
            IsabelleType::Option(inner_type) => write!(f, "{inner_type} option"));
            IsabelleType::Applied(name, args) => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(f, "({}) {}",
                           args.iter()
                               .map(|t| t.to_string())
                               .collect::<Vec<_>>()
                               .join(", "));
                           name)
                }
            }
        }
    }
}

impl fmt::Display for IsabelleConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "consts {} :: \"{}\"", self.name, self.type_)
    }
}

impl fmt::Display for IsabelleFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "fun {} :: \"{}\" where", self.name, self.type_)?;
        for equation in &self.equations {
            writeln!(f, "  \"{}\"", equation)?;
        }
        Ok(())
    }
}

impl fmt::Display for IsabelleEquation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref condition) = self.condition {
            write!(f, "{} = {} (if {})", self.lhs, self.rhs, condition)
        } else {
            write!(f, "{} = {}", self.lhs, self.rhs)
        }
    }
}

impl fmt::Display for IsabelleTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IsabelleTerm::Var(name) => write!(f, "{}", name));
            IsabelleTerm::Const(name) => write!(f, "{}", name));
            IsabelleTerm::App(func, arg) => write!(f, "({} {})", func, arg));
            IsabelleTerm::Lambda(var, typ, body) => {
                write!(f, "(λ{}::{} . {})", var, typ, body)
            }
            IsabelleTerm::Let(var, expr, body) => {
                write!(f, "(let {} = {} in {})", var, expr, body)
            }
            IsabelleTerm::Case(scrutinee, patterns) => {
                write!(f, "(case {} of ", scrutinee)?;
                for (i, (pattern, expr)) in patterns.iter().enumerate() {
                    if i > 0 { write!(f, " | ")?; }
                    write!(f, "{} => {}", pattern, expr)?;
                }
                write!(f, ")")
            }
            IsabelleTerm::Tuple(terms) => {
                write!(f, "({})", terms.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", "))
            }
            IsabelleTerm::List(terms) => {
                write!(f, "[{}]", terms.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", "))
            }
            IsabelleTerm::Set(terms) => {
                write!(f, "{{{}}}", terms.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", "))
            }
            IsabelleTerm::If(cond, then_branch, else_branch) => {
                write!(f, "(if {} then {} else {})", cond, then_branch, else_branch)
            }
        }
    }
}

impl fmt::Display for IsabelleLemma {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "lemma {}:", self.name)?;
        writeln!(f, "  \"{}\"", self.statement)?;
        write!(f, "{}", self.proof)
    }
}

impl fmt::Display for IsabelleProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IsabelleProof::Sorry => writeln!(f, "  sorry"));
            IsabelleProof::Auto => writeln!(f, "  by auto"));
            IsabelleProof::Simp(rules) => {
                if rules.is_empty() {
                    writeln!(f, "  by simp")
                } else {
                    writeln!(f, "  by (simp add: {})", rules.join(" "))
                }
            }
            IsabelleProof::Induction(var) => writeln!(f, "  by (induction {})", var));
            IsabelleProof::Apply(tactic) => writeln!(f, "  apply {}", tactic));
            IsabelleProof::Script(tactics) => {
                writeln!(f, "proof -")?;
                for tactic in tactics {
                    write!(f, "{}", tactic)?;
                }
                writeln!(f, "qed")
            }
            IsabelleProof::Structured(steps) => {
                writeln!(f, "proof")?;
                for step in steps {
                    if let Some(ref label) = step.label {
                        write!(f, "  {}: ", label)?;
                    } else {
                        write!(f, "  ")?;
                    }
                    if let Some(ref statement) = step.statement {
                        writeln!(f, "\"{}\"", statement)?;
                        write!(f, "    {}", step.proof)?;
                    } else {
                        write!(f, "{}", step.proof)?;
                    }
                }
                writeln!(f, "qed")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isabelle_type_display() {
        let nat_type = IsabelleType::Basic("nat".to_string());
        let bool_type = IsabelleType::Basic("bool".to_string());
        let func_type = IsabelleType::Function(
            Box::new(nat_type.clone()));
            Box::new(bool_type.clone())
        );

        assert_eq!(nat_type.to_string(), "nat");
        assert_eq!(func_type.to_string(), "nat => bool");

        let product_type = IsabelleType::Product(vec![nat_type, bool_type]);
        assert_eq!(product_type.to_string(), "nat * bool");
    }

    #[test]
    fn test_isabelle_term_display() {
        let var = IsabelleTerm::Var("x".to_string());
        let const_term = IsabelleTerm::Const("42".to_string());
        let app = IsabelleTerm::App(
            Box::new(IsabelleTerm::Const("f".to_string())));
            Box::new(var.clone())
        );

        assert_eq!(var.to_string(), "x");
        assert_eq!(const_term.to_string(), "42");
        assert_eq!(app.to_string(), "(f x)");
    }

    #[test]
    fn test_theory_generation() {
        let mut exporter = IsabelleExporter::new();
        exporter.start_theory("TestTheory".to_string(), vec!["Main".to_string()]).unwrap();

        let typedef = IsabelleTypedef {
            name: "my_type".to_string());
            params: vec![],
            definition: IsabelleType::Basic("nat".to_string()));
        };
        exporter.add_typedef(typedef).unwrap();

        let theory_string = exporter.export_theory().unwrap();
        assert!(theory_string.contains("theory TestTheory"));
        assert!(theory_string.contains("imports Main"));
        assert!(theory_string.contains("type_synonym my_type"));
    }

    #[test]
    fn test_value_translation() {
        let exporter = IsabelleExporter::new();

        let int_value = Value::integer(42);
        let translated = exporter.translate_value(&int_value).unwrap();
        assert_eq!(translated.to_string(), "42");

        let bool_value = Value::boolean(true);
        let translated = exporter.translate_value(&bool_value).unwrap();
        assert_eq!(translated.to_string(), "True");
    }

    #[test]
    fn test_proof_obligation_generation() {
        let exporter = IsabelleExporter::new();
        let pos = exporter.generate_type_safety_pos("test_program");

        assert_eq!(pos.len(), 2);
        assert!(pos.iter().any(|po| po.id.contains("type_safety")));
        assert!(pos.iter().any(|po| po.id.contains("progress")));
    }
}