<!--
Sync Impact Report:
- Version change: 1.0.0 → 1.1.0
- Modified principles: None
- Added sections:
  * Development Workflow › Constitution Check Gates
- Removed sections: None
- Templates requiring updates:
  ✅ .specify/templates/plan-template.md (Constitution gates synced)
- Follow-up TODOs: None
-->

# Loco Framework Constitution

## Core Principles

### I. Simplicity & DRY (Don't Repeat Yourself) - NON-NEGOTIABLE

Every component MUST follow the DRY principle - avoid duplicating logic, templates, or generation code across multiple modules. When functionality exists in a core library (e.g., loco-gen), integration projects (e.g., loco-bindings) MUST NOT reimplement that functionality.

**Rationale**: The loco-bindings refactoring demonstrated an 83% code reduction (from ~1500 lines to ~250 lines) by eliminating duplicate implementations of loco-gen functionality. This massive simplification improved maintainability, reduced bugs, and ensured consistency.

**Rules**:
- Code MUST NOT duplicate existing functionality from core libraries
- When integration is needed, create thin binding layers (~200-300 lines max)
- Reject complexity that can be solved by better code organization
- Question every new module: "Does this already exist elsewhere?"

### II. Thin Binding Layers

Integration code (Python bindings, FFI, language bindings) MUST be thin wrapper layers that handle ONLY type conversions and language-specific interfaces. All business logic, validation, and generation MUST remain in the core Rust library.

**Rationale**: Thin binding layers ensure single source of truth, automatic synchronization with core improvements, and minimal maintenance burden. The loco-bindings layer demonstrates this with just two files: `lib.rs` (235 lines) and `error.rs` (14 lines).

**Rules**:
- Binding code MUST NOT contain business logic
- Binding code MUST NOT duplicate validation or generation logic
- Binding code responsibility: Type conversion + Error translation only
- Core logic stays in Rust; bindings expose it to other languages

### III. Direct Integration Over Re-implementation

When extending Loco functionality to new interfaces (CLI, MCP, Python, etc.), projects MUST use direct function calls to loco-gen rather than reimplementing templates, field parsing, or file operations.

**Rationale**: Direct integration guarantees 100% compatibility with the Loco CLI, eliminates version drift, and ensures bug fixes propagate automatically to all consumers.

**Rules**:
- MUST use loco-gen's `Component` enum and `generate()` function
- MUST use loco-gen's field parsing and validation
- MUST use loco-gen's template engine (no custom templates)
- Integration projects provide user-facing APIs only

### IV. Maintainability First

Code structure MUST prioritize long-term maintainability over short-term convenience. Prefer small, focused modules that do one thing well. When refactoring significantly improves maintainability, execute the refactoring even if the current code "works".

**Rationale**: The loco-bindings refactoring was triggered by recognizing maintenance debt - two codebases doing the same thing means double the bug fixes, double the template updates, and double the testing burden.

**Rules**:
- Each module has a single, clear responsibility
- Dependencies MUST be justified (remove unused deps)
- Code reduction is valuable - fewer lines = less to maintain
- Refactor when debt is identified, don't wait for breakage

### V. Performance Through Native Code

Loco is built on Rust for performance. Integration layers MUST NOT introduce significant overhead. Avoid caching layers, thread pools, or optimization code in bindings - let the core Rust implementation handle performance.

**Rationale**: The refactoring removed tokio, once_cell, and lru caching from loco-bindings because loco-gen already handles performance efficiently. Extra layers added complexity without measurable benefit.

**Rules**:
- No caching in binding layers (core handles it)
- No custom thread pools in bindings
- No "performance optimizations" without benchmarks showing need
- Trust Rust's native performance

## Development Workflow

### Testing Requirements

- **Unit Tests**: Required for core logic in loco-gen
- **Integration Tests**: Required for binding layers to verify correct invocation
- **Contract Tests**: Required when exposing new external APIs
- **Test-First**: Write tests before implementation when adding new features

### Code Review Standards

- **Simplicity Check**: Can this be simpler? Is any code duplicated?
- **DRY Verification**: Does this reimplementation already exist in a core library?
- **Dependency Audit**: Are all dependencies necessary? Can we remove any?
- **Performance Justification**: Are optimizations backed by profiling data?

### Documentation Standards

- **README**: MUST explain architecture (thin layer vs. core logic separation)
- **API Documentation**: MUST document public functions and error handling
- **Refactoring Documentation**: MUST document major refactorings with before/after metrics
- **Example Code**: MUST provide working examples for all public APIs

### Constitution Check Gates

Each feature proposal MUST pass these gates before work begins and after significant design updates.

- **G1 Simplicity & DRY**: Demonstrate reuse of existing core capabilities and eliminate duplicate logic.
- **G2 Thin Binding Layers**: Keep integration layers limited to type conversion and interface glue with zero business logic.
- **G3 Direct Integration**: Route all generation, validation, and templating through loco-gen without local substitutes.
- **G4 Maintainability First**: Show that module boundaries remain focused, dependencies justified, and debt reduction opportunities captured.
- **G5 Performance Through Native Code**: Avoid adding caches, thread pools, or speculative optimizations outside the Rust core unless backed by benchmarks.
- **G6 Workflow Discipline**: Confirm test-first sequencing, documentation updates, and compliance review steps are planned per Development Workflow standards.

## Governance

### Amendment Process

This constitution can be amended through:
1. Proposal documenting the principle change and rationale
2. Review by core maintainers
3. Approval requiring demonstration of value (metrics, examples)
4. Migration plan for existing code violating new principles

### Compliance Review

- All PRs MUST verify compliance with constitutional principles
- Significant complexity MUST be justified against simplicity principle
- Code reviews MUST check for DRY violations and duplicate implementations
- Quarterly audits identify technical debt and refactoring opportunities

### Living Document

This constitution reflects learnings from real refactoring experiences (loco-bindings 2025-10-04). As the project evolves, principles may be added, refined, or amended based on demonstrated value.

**Version**: 1.1.0 | **Ratified**: 2025-10-04 | **Last Amended**: 2025-10-08
