# Formal Methods Integration Setup

This document provides installation and configuration instructions for Lambdust's formal methods integration features.

## Overview

Lambdust's formal methods integration provides translation between:
- **B-Method** specifications → Lambdust functional programs
- **Event-B** machines → Lambdust actor systems  
- **Isabelle/HOL** theories → Mechanical proof verification

All formal methods features are **disabled by default** and require explicit activation through feature flags.

## Feature Flags

```toml
[dependencies]
lambdust = { version = "0.1.1", features = ["formal-methods"] }
```

Available feature flags:
- `formal-methods` - Basic formal methods infrastructure
- `event-b` - Event-B machine translation 
- `b-method` - B-Method specification support
- `isabelle-hol` - Isabelle/HOL proof integration
- `certified-translation` - Complete formal verification stack

## Tool Requirements

### For Event-B Integration
- **Rodin Platform** (https://www.event-b.org/)
- **ProB** model checker (https://prob.hhu.de/)

### For B-Method Integration  
- **Atelier B** (https://www.atelierb.eu/)
- **B Toolkit** (optional, for legacy support)

### For Isabelle/HOL Integration
- **Isabelle** (https://isabelle.in.tum.de/)
  - Version 2023 or later recommended
  - With HOL library extensions

## Installation Steps

### 1. Install External Tools

#### Isabelle/HOL
```bash
# Download from official website
wget https://isabelle.in.tum.de/dist/Isabelle2023_linux.tar.gz
tar xzf Isabelle2023_linux.tar.gz
export PATH=$PATH:/path/to/Isabelle2023/bin
```

#### Event-B (Rodin)
```bash
# Download from event-b.org
# Follow platform-specific installation instructions
# Ensure ProB is installed as a plugin
```

#### B-Method (Atelier B)
```bash
# Commercial tool - requires license
# Follow vendor installation instructions
```

### 2. Configure Environment

```bash
# Set environment variables
export ISABELLE_HOME=/path/to/Isabelle2023
export PROB_HOME=/path/to/prob
export ATELIER_B_HOME=/path/to/atelierb
```

### 3. Enable Features in Cargo.toml

```toml
[dependencies]
lambdust = { 
    version = "0.1.1", 
    features = ["certified-translation"] 
}
```

## Usage Examples

### Basic B-Method Translation
```rust
use lambdust::formal::BMethodToLambdustTranslator;

let translator = BMethodToLambdustTranslator::new();
let lambdust_code = translator.translate_machine(&b_machine)?;
```

### Event-B Machine Translation
```rust
use lambdust::formal::EventBToLambdustTranslator;

let translator = EventBToLambdustTranslator::new();
let actor_system = translator.translate_machine(&eventb_machine)?;
```

### Isabelle/HOL Proof Generation
```rust
use lambdust::formal::IsabelleExporter;

let mut exporter = IsabelleExporter::new();
exporter.start_theory("MyTheory", vec!["Main"]);
let theory = exporter.export_theory()?;
```

## Verification Workflow

1. **Translate** formal specification to Lambdust
2. **Generate** proof obligations
3. **Export** to Isabelle/HOL for mechanical verification
4. **Obtain** translation certificate with formal guarantees

## Troubleshooting

### Common Issues

1. **Tool not found**: Ensure all external tools are in PATH
2. **Version incompatibility**: Check tool version requirements
3. **License issues**: Verify commercial tool licenses

### Support

For formal methods integration issues:
- Check tool-specific documentation
- Verify environment variable configuration
- Ensure all dependencies are properly installed

## References

- [Event-B Official Documentation](https://www.event-b.org/)
- [Isabelle/HOL Manual](https://isabelle.in.tum.de/documentation.html)
- [B-Method Resources](https://www.atelierb.eu/documentation/)
- [Lambdust Formal Semantics](./LambdustFormalSemantics.md)