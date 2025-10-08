# Tasks: Expand MCP Tools via Loco Bindings

**Input**: `/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/plan.md`

**Prerequisites**: `/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/research.md`, `/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/data-model.md`, `/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/contracts/cli-utilities.inventory.yaml`, `/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/quickstart.md`

## Phase 3.1: Setup
- [X] T001 Prep Python tooling: run `uv sync` (or `pip install -e`) for `/Users/devel0per/Code/framework/loco/loco-mcp/pyproject.toml`, configure `.env` automation credentials, and document any missing dependencies in `/Users/devel0per/Code/framework/loco/loco-mcp/README.md`.
- [X] T002 Verify Rust workspace readiness: execute `cargo check -p loco-cli -p loco-bindings` from `/Users/devel0per/Code/framework/loco/Cargo.toml` to capture baseline artifacts and note required features for later bindings`. (Executed `cargo check -p loco-rs` due to workspace package naming.)

## Phase 3.2: Tests First (write → watch fail)
- [X] T003 [P] Add failing contract test ensuring every entry in `/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/contracts/cli-utilities.inventory.yaml` exposes owner, approvals, timeout, and dependencies in `/Users/devel0per/Code/framework/loco/loco-mcp/loco-mcp-server/tests/test_inventory_contract.py`.
- [X] T004 Extend `/Users/devel0per/Code/framework/loco/loco-mcp/loco-mcp-server/tests/test_tools.py` with failing tests asserting `list_tools` advertises `migrate_db`, `rotate_keys`, `clean_temp` schemas and that `call_tool` forwards validated arguments.
- [X] T005 Create failing end-to-end scenarios in `/Users/devel0per/Code/framework/loco/loco-mcp/loco-mcp-server/tests/test_scenarios.py` that simulate migrate→rotate→clean flows, compare CLI parity outputs, and enforce required approvals ordering.
- [X] T006 [P] Introduce failing audit log test in `/Users/devel0per/Code/framework/loco/loco-mcp/loco-mcp-server/tests/test_audit_log.py` verifying tool invocations append parameter hashes to `/var/log/loco-mcp/audit.log`.
- [X] T007 [P] Add failing timeout enforcement tests in `/Users/devel0per/Code/framework/loco/loco-mcp/loco-mcp-server/tests/test_timeouts.py` covering default 60s and override 300s behaviour.

## Phase 3.3: Core Implementation (after T003-T007 fail)
- [X] T008 Expose programmatic entrypoints for `Db::migrate`, `Task`, and scheduler/job flows by refactoring `/Users/devel0per/Code/framework/loco/src/cli.rs`, `/Users/devel0per/Code/framework/loco/src/task.rs`, and `/Users/devel0per/Code/framework/loco/src/scheduler.rs` to provide reusable functions without Clap context.
- [X] T009 Extend PyO3 bindings in `/Users/devel0per/Code/framework/loco/loco-mcp/loco-bindings/src/lib.rs` (and supporting modules) to call the new CLI entrypoints with argument validation and structured results.
- [X] T010 Implement asynchronous CLI bridge helpers in `/Users/devel0per/Code/framework/loco/loco-mcp/loco-mcp-server/src/tools.py` to invoke new bindings, support environment selection, and convert outputs for MCP replies.
- [X] T011 [P] Register MCP Tool Listing metadata in `/Users/devel0per/Code/framework/loco/loco-mcp/loco-mcp-server/src/server.py`, covering schemas, guardrails, and environment parameter exposure for the three tools.
- [X] T012 [P] Update `/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/contracts/cli-utilities.inventory.yaml` to fully populate `CLI Utility Profile` entities with timeout defaults, dependency notes, and approval sequences.
- [X] T013 [P] Implement `Execution Assurance Record` persistence utilities in `/Users/devel0per/Code/framework/loco/loco-mcp/loco-mcp-server/src/validation.py`, recording checksum comparisons and operator metadata.

## Phase 3.4: Integration & Safety
- [X] T014 Wire audit logging and parameter hashing by updating `/Users/devel0per/Code/framework/loco/loco-mcp/loco-mcp-server/src/security.py` (and calling sites) so each tool invocation writes structured JSON to `/var/log/loco-mcp/audit.log`.
- [X] T015 Add timeout and environment configuration surfaces in `/Users/devel0per/Code/framework/loco/loco-mcp/loco-mcp-server/src/config.py`, ensuring bridge helpers honour overrides and fallbacks.
- [X] T016 Expand multi-step orchestration in `/Users/devel0per/Code/framework/loco/loco-mcp/loco-mcp-server/src/tools.py` to chain scheduler/jobs commands for rotate/clean workflows while remaining interruptible.
- [X] T017 Execute regression suites: run `cargo test -p loco-cli` and `pytest` inside `/Users/devel0per/Code/framework/loco/loco-mcp` capturing evidence for Execution Assurance updates.

## Phase 3.5: Polish & Documentation
- [X] T018 [P] Refresh `/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/quickstart.md` with MCP invocation steps, parity checklist, and audit verification guidance.
- [X] T019 [P] Update `/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/data-model.md` to reflect final field defaults (timeouts, guardrails) for all entities.
- [X] T020 [P] Document new MCP tools and operational runbooks in `/Users/devel0per/Code/framework/loco/loco-mcp/docs/API.md` and `/Users/devel0per/Code/framework/loco/loco-mcp/README.md`.

## Dependencies
- T003–T007 depend on setup tasks T001–T002.
- T008 depends on T003–T007 (tests must exist and fail before refactor).
- T009 depends on T008.
- T010 depends on T009.
- T011 depends on T010 and T012 (manifest fields required for schema alignment).
- T012 depends on T003 (contract expectations) and completes before T011.
- T013 depends on T010 and T011.
- T014–T016 depend on T008–T013.
- T017 requires all implementation and integration tasks (T008–T016) to finish.
- T018–T020 depend on T017 for validated behaviour.

## Parallel Execution Examples
```bash
# Contract and safety tests in parallel once setup is done
task exec T003
task exec T006
task exec T007

# Entity documentation updates together after core bindings land
task exec T012
task exec T018
task exec T019

# Final documentation sweep in parallel
task exec T018
task exec T019
task exec T020
```

## Validation Checklist
- [ ] All contract files mapped to explicit tests (T003).
- [ ] All entities from `data-model.md` have implementation tasks (T011–T013) and documentation updates (T018–T019).
- [ ] Tests (T003–T007) precede implementation tasks (T008–T017).
- [ ] [P] tasks touch distinct files and honour dependencies.
- [ ] Each task references absolute paths and concrete commands where applicable.
- [ ] Execution Assurance evidence captured via T013 and T017.
- [ ] Quickstart and docs refreshed before completion (T018–T020).

