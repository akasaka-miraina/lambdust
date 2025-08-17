# Architecture & Design Documentation

This directory contains architecture and design documents for the Lambdust R7RS-large Scheme interpreter.

## Contents

### Core Architecture
- **[jit_architecture_design.md](jit_architecture_design.md)** - JIT compilation system architecture and design
- **[dependent_type_theory.md](dependent_type_theory.md)** - Dependent type system implementation and theory
- **[DOMAIN_DRIVEN_VALUE_OPTIMIZATION_ARCHITECTURE.md](DOMAIN_DRIVEN_VALUE_OPTIMIZATION_ARCHITECTURE.md)** - Value optimization domain-driven design
- **[TECHNICAL_IMPLEMENTATION_ROADMAP.md](TECHNICAL_IMPLEMENTATION_ROADMAP.md)** - Technical implementation roadmap and milestones

### Formal Semantics
- **[LambdustFormalSemantics.md](LambdustFormalSemantics.md)** - Formal semantics specification
- **[LambdustFormalSemantics.tex](LambdustFormalSemantics.tex)** - LaTeX source for formal semantics
- **[FormalSemantics_v020_Design.md](FormalSemantics_v020_Design.md)** - Version 0.2.0 formal semantics design
- **[termination_system.md](termination_system.md)** - Termination analysis and guarantees

### Formal Methods Integration
- **[FormalMethodsSetup.md](FormalMethodsSetup.md)** - Setup guide for formal verification tools
- **[EventB_BMethod_Equivalence.md](EventB_BMethod_Equivalence.md)** - Event-B and B-Method correspondence
- **[isabelle_hol/](isabelle_hol/)** - Isabelle/HOL formal verification artifacts

## Purpose

These documents define:
- System architecture and design decisions
- Formal semantics and mathematical foundations
- Verification methodologies and formal proofs
- Type system theoretical foundations

## Audience

- System architects
- Language designers
- Formal methods researchers
- Academic researchers working with Lambdust