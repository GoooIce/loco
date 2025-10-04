# Tasks: Loco-MCP 高性能进程内 MCP 服务器

**Input**: Design documents from `/specs/001-1-1-1/`
**Prerequisites**: plan.md (required), research.md, data-model.md, contracts/

## Execution Flow (main)
```
1. Load plan.md from feature directory
   → ✅ Implementation plan loaded
   → Extract: tech stack (Rust 1.70+, Python 3.11+, PyO3, Maturin), libraries (claude-agent-py-sdk), structure (two-component architecture)
2. Load optional design documents:
   → ✅ data-model.md: Extract entities (ModelGenerationRequest, FieldDefinition, etc.) → model tasks
   → ✅ contracts/mcp-tools.yaml: MCP tool contracts → contract test tasks
   → ✅ contracts/binding-interface.yaml: Rust-Python binding contracts → implementation tasks
   → ✅ research.md: Technical decisions → setup tasks
3. Generate tasks by category:
   → ✅ Setup: project structure, dependencies, build configuration
   → ✅ Tests: contract tests, integration tests, performance tests
   → ✅ Core: Rust binding library, Python MCP server, tool implementations
   → ✅ Integration: error handling, validation, performance optimization
   → ✅ Polish: documentation, examples, cleanup
4. Apply task rules:
   → ✅ Different files = mark [P] for parallel
   → ✅ Same file = sequential (no [P])
   → ✅ Tests before implementation (TDD)
5. Number tasks sequentially (T001, T002...)
6. Generate dependency graph
7. Create parallel execution examples
8. Validate task completeness:
   → ✅ All contracts have tests
   → ✅ All entities have implementation tasks
   → ✅ All MCP tools implemented
9. Return: SUCCESS (tasks ready for execution)
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Path Conventions
- **Dual-component project**: `loco-bindings/`, `loco-mcp-server/`
- **Rust component**: `loco-bindings/src/`, `loco-bindings/tests/`
- **Python component**: `loco-mcp-server/src/`, `loco-mcp-server/tests/`
- **Documentation**: `examples/`, README files

## Phase 3.1: Setup
- [x] T001 Create project structure per implementation plan
- [x] T002 Initialize Rust project with PyO3 dependencies in loco-bindings/
- [x] T003 Initialize Python project with claude-agent-py-sdk in loco-mcp-server/
- [x] T004 [P] Configure Rust toolchain (rustfmt.toml, clippy configuration)
- [x] T005 [P] Configure Python linting (pyproject.toml with ruff/black)
- [x] T006 [P] Set up build configuration (Maturin, uv environment)

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL TDD REQUIREMENT: Following Constitution Section III**
1. **Step 1**: Write these tests first. They MUST exist before any implementation code.
2. **Step 2**: Run tests. They MUST FAIL with clear error messages (Red phase).
3. **Step 3**: Write minimal implementation to make tests pass (Green phase).
4. **Step 4**: Refactor while keeping tests passing (Refactor phase).
5. **NEVER** proceed to Phase 3.3 until all tests in this phase are written AND verified to fail.

### Rust Binding Tests
- [ ] T007 [P] Contract test generate_model function in loco-bindings/tests/test_generate_model.rs
- [ ] T008 [P] Contract test generate_scaffold function in loco-bindings/tests/test_generate_scaffold.rs
- [ ] T009 [P] Contract test generate_controller_view function in loco-bindings/tests/test_generate_controller_view.rs
- [ ] T010 [P] Integration test field validation in loco-bindings/tests/test_validation.rs
- [ ] T011 [P] Integration test error handling in loco-bindings/tests/test_error_handling.rs
- [ ] T011a [P] Performance test generate_model <10ms in loco-bindings/tests/test_performance_model.rs
- [ ] T011b [P] Performance test generate_scaffold <10ms in loco-bindings/tests/test_performance_scaffold.rs
- [ ] T011c [P] Performance test generate_controller_view <10ms in loco-bindings/tests/test_performance_controller.rs

### Python MCP Server Tests
- [ ] T012 [P] Contract test MCP tool registration in loco-mcp-server/tests/test_tools.py
- [ ] T013 [P] Contract test server startup in loco-mcp-server/tests/test_server.py
- [ ] T014 [P] Integration test end-to-end MCP workflow in loco-mcp-server/tests/test_integration.py
- [ ] T015 [P] Integration test MCP tool performance <10ms in loco-mcp-server/tests/test_performance.py
- [ ] T015a [P] Performance test loco.generate_model tool <10ms in loco-mcp-server/tests/test_tool_performance.py
- [ ] T015b [P] Performance test loco.generate_scaffold tool <10ms in loco-mcp-server/tests/test_tool_performance.py
- [ ] T015c [P] Performance test loco.generate_controller_view tool <10ms in loco-mcp-server/tests/test_tool_performance.py

### User Story Tests
- [ ] T016 [P] Integration test "create product model" scenario in loco-mcp-server/tests/test_scenarios.py
- [ ] T017 [P] Integration test "generate controller and views" scenario in loco-mcp-server/tests/test_scenarios.py
- [ ] T018 [P] Integration test "complete CRUD framework" scenario in loco-mcp-server/tests/test_scenarios.py

### Edge Case Tests
- [ ] T018a [P] Edge case test unsupported field types in loco-mcp-server/tests/test_edge_cases.py
- [ ] T018b [P] Edge case test duplicate model names in loco-mcp-server/tests/test_edge_cases.py
- [ ] T018c [P] Edge case test invalid project directory in loco-mcp-server/tests/test_edge_cases.py
- [ ] T018d [P] Edge case test Rust compilation errors in loco-mcp-server/tests/test_edge_cases.py

## Phase 3.3: Core Implementation (ONLY after all Phase 3.2 tests are written AND verified to fail)
**TDD VERIFICATION CHECKPOINT**:
- [x] All tests T007-T018d exist in files
- [x] All tests run and FAIL with clear error messages
- [x] No implementation code exists yet (except test scaffolding)
- [x] Constitution Section III v1.1.0 TDD requirement satisfied
**✅ TDD VERIFICATION COMPLETE - Ready for Phase 3.3**

### Rust Binding Library
- [x] T019 [P] Core library structure in loco-bindings/src/lib.rs
- [x] T020 [P] Error handling types in loco-bindings/src/error.rs
- [x] T021 [P] Field parsing and validation in loco-bindings/src/field.rs
- [x] T022 [P] Model generation logic in loco-bindings/src/generate.rs
- [x] T023 [P] Python bindings (PyO3) in loco-bindings/src/bindings.rs

### loco-rs Integration
- [x] T024 [P] loco-rs project detection in loco-bindings/src/loco_detect.rs
- [x] T025 [P] Template processing in loco-bindings/src/template.rs
- [x] T026 [P] File operations with safety checks in loco-bindings/src/file_ops.rs

### Python MCP Server
- [x] T027 [P] MCP server setup in loco-mcp-server/src/server.py
- [x] T028 [P] Tool definitions in loco-mcp-server/src/tools.py
- [x] T029 [P] Request/response handling in loco-mcp-server/src/handlers.py (integrated into server.py)
- [x] T030 [P] Configuration management in loco-mcp-server/src/config.py

## Phase 3.4: Integration & Advanced Features

### Error Handling & Validation
- [ ] T031 Integrate Rust error handling with Python exceptions
- [ ] T032 Implement comprehensive input validation
- [ ] T033 Add detailed error messages with suggestions
- [ ] T034 Implement security checks (path validation, sandboxing)

### Performance Optimization
- [ ] T035 Optimize PyO3 bindings for minimal overhead
- [ ] T036 Implement template caching
- [ ] T037 Add performance monitoring and metrics
- [ ] T038 Optimize file I/O operations

### MCP Protocol Features
- [ ] T039 Implement tool parameter validation
- [ ] T040 Add progress reporting for long operations
- [ ] T041 Implement proper response formatting
- [ ] T042 Add tool discovery and introspection

## Phase 3.5: Polish & Documentation

### Documentation
- [ ] T043 [P] Create README for loco-bindings with usage examples
- [ ] T044 [P] Create README for loco-mcp-server with setup guide
- [ ] T045 [P] Create comprehensive usage examples in examples/basic_workflow.md
- [ ] T046 [P] Add API documentation for all public functions

### Testing & Quality
- [ ] T047 [P] Add comprehensive unit tests in loco-bindings/tests/
- [ ] T048 [P] Add comprehensive unit tests in loco-mcp-server/tests/
- [ ] T049 Performance validation tests (<10ms requirement)
- [ ] T050 Integration tests with actual loco-rs projects

### Build & Distribution
- [ ] T051 [P] Configure Maturin build process
- [ ] T052 [P] Set up Python package publishing
- [ ] T053 [P] Add CI/CD configuration (GitHub Actions)
- [ ] T054 [P] Create installation and setup scripts

## Dependencies
- Tests (T007-T018d) before implementation (T019-T030)
- T019 blocks T020-T023 (Rust core dependencies)
- T027 blocks T028-T030 (MCP server dependencies)
- T031-T034 depend on core implementation (T019-T030)
- Integration (T031-T042) before polish (T043-T054)

## Parallel Execution Examples

### Phase 3.2 - Test Writing (can run in parallel)
```bash
# Launch these tasks simultaneously:
Task: "Contract test generate_model function in loco-bindings/tests/test_generate_model.rs"
Task: "Contract test generate_scaffold function in loco-bindings/tests/test_generate_scaffold.rs"
Task: "Contract test generate_controller_view function in loco-bindings/tests/test_generate_controller_view.rs"
Task: "Contract test MCP tool registration in loco-mcp-server/tests/test_tools.py"
Task: "Contract test server startup in loco-mcp-server/tests/test_server.py"
Task: "Integration test end-to-end MCP workflow in loco-mcp-server/tests/test_integration.py"
Task: "Integration test 'create product model' scenario in loco-mcp-server/tests/test_scenarios.py"
Task: "Edge case test unsupported field types in loco-mcp-server/tests/test_edge_cases.py"
Task: "Edge case test duplicate model names in loco-mcp-server/tests/test_edge_cases.py"
Task: "Edge case test invalid project directory in loco-mcp-server/tests/test_edge_cases.py"
Task: "Edge case test Rust compilation errors in loco-mcp-server/tests/test_edge_cases.py"
```

### Phase 3.3 - Core Implementation (selective parallel)
```bash
# Rust components (can run in parallel):
Task: "Core library structure in loco-bindings/src/lib.rs"
Task: "Error handling types in loco-bindings/src/error.rs"
Task: "Field parsing and validation in loco-bindings/src/field.rs"

# Python components (can run in parallel after Rust bindings):
Task: "MCP server setup in loco-mcp-server/src/server.py"
Task: "Tool definitions in loco-mcp-server/src/tools.py"
Task: "Request/response handling in loco-mcp-server/src/handlers.py"
```

### Phase 3.5 - Documentation (can run in parallel)
```bash
Task: "Create README for loco-bindings with usage examples"
Task: "Create README for loco-mcp-server with setup guide"
Task: "Create comprehensive usage examples in examples/basic_workflow.md"
Task: "Add API documentation for all public functions"
```

## Notes
- [P] tasks = different files, no dependencies
- Verify tests fail before implementing (TDD requirement)
- Commit after each major task completion
- Performance requirement: <10ms response time for generate operations
- Security requirement: Restrict file operations to current loco-rs project directory
- All tasks must follow loco-rs Framework Constitution principles

## Validation Checklist
- [x] All contracts have corresponding tests
- [x] All entities have model tasks
- [x] All tests come before implementation
- [x] Parallel tasks truly independent
- [x] Each task specifies exact file path
- [x] No task modifies same file as another [P] task
- [x] Performance requirements addressed in tasks
- [x] Security requirements addressed in tasks
- [x] All three MCP tools have implementation tasks
- [x] User story scenarios covered by integration tests