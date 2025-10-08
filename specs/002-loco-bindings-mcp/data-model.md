# Data Model: Expand MCP Tools via Loco Bindings

## Entities

### CLI Utility Profile
- **Description**: Authoritative record for each CLI command eligible for MCP conversion.
- **Fields**:
  - `id` (string, required): Unique identifier (snake_case).
  - `name` (string, required): Human-readable title.
  - `description` (string, required): Business summary of command purpose.
  - `owner` (string, required): Accountability team or individual.
  - `usage_frequency` (enum: hourly|daily|weekly|monthly|ad-hoc, required): Operational cadence used for prioritization.
  - `risk` (enum: low|medium|high|critical, required): Operational risk category.
  - `compliance_notes` (string, optional): Policy annotations (SOX, PCI, etc.).
  - `automation_coverage` (percentage 0-100, required): Current automation maturity.
  - `priority` (integer >=1, required): Migration order within backlog.
  - `timeout` (integer 10-300, required): Default execution timeout in seconds.
  - `dependencies` (string[], optional): External services or resources touched.
  - `approvals` (string[], required): Roles that must sign-off prior to MCP launch.
  - `mcp_tool_name` (string, required): Corresponding MCP tool identifier.
  - `default_environment` (string, required): Default environment for execution.
  - `guardrails` (string[], required): Safety constraints and validation rules.
  - `audit_log_path` (string, required): Location for execution audit entries.

### MCP Tool Listing
- **Description**: MCP-facing representation including prompts, guardrails, and metadata.
- **Fields**:
  - `tool_name` (string, required): Registered MCP tool identifier.
  - `cli_id` (foreign key to CLI Utility Profile, required): Connects listing to source CLI command.
  - `description` (string, required): Assistant-facing description.
  - `inputs` (array of InputParameter, required): Typed parameters required by the tool.
  - `outputs` (array of OutputDescriptor, required): Expected structured outputs or files.
  - `prompt_guidance` (string, optional): Step-by-step instructions for interactive flows.
  - `timeout_seconds` (integer 10-300, required): Execution timeout per FR-009. Default: 60.
  - `ownership` (string, required): Responsible team reference (mirrors CLI owner).
  - `last_verified` (datetime, required): Timestamp of parity validation.
  - `guardrails` (string[], required): Safety constraints (e.g., "只读模式", "非高峰时段执行").
  - `audit_log_path` (string, required): Location for execution audit entries. Default: "/var/log/loco-mcp/audit.log".
  - `environment_override` (string, optional): Environment-specific configuration override.
  - `approval_sequence` (string[], required): Required approval roles in execution order.

### Execution Assurance Record
- **Description**: Evidence that MCP tools match CLI behaviors and remain compliant.
- **Fields**:
  - `cli_id` (foreign key to CLI Utility Profile, required): Source CLI command identifier.
  - `verification_run_id` (string, required): Unique identifier for this validation run.
  - `expected_checksum` (string, required): Hash of canonical output from CLI baseline run.
  - `actual_checksum` (string, required): Result from MCP invocation during validation.
  - `variance_notes` (string, optional): Explanation when checksums differ.
  - `tester` (string, required): Operator performing the validation.
  - `run_timestamp` (datetime, required): When the validation was performed.
  - `status` (enum: pass|fail|waived, required): Validation result status.
  - `execution_time_ms` (integer, required): Execution time in milliseconds.
  - `environment` (string, required): Environment used for validation.
  - `approvals_verified` (string[], required): Approval roles that were validated.
  - `audit_log_entry` (string, required): Reference to audit log entry for this run.

### InputParameter (Embedded)
- `name` (string, required): Parameter name in snake_case.
- `type` (enum: string|integer|boolean|enum|file, required): Data type for validation.
- `required` (boolean, required): Whether parameter is mandatory.
- `description` (string, required): Human-readable parameter description.
- `enum_values` (string[], optional when type=enum): Valid values for enum parameters.
- `default_value` (string, optional): Default value if not provided.
- `validation_pattern` (string, optional): Regex pattern for string validation.
- `min_value` (integer, optional): Minimum value for integer parameters.
- `max_value` (integer, optional): Maximum value for integer parameters.

### OutputDescriptor (Embedded)
- `name` (string, required): Output field name.
- `type` (enum: text|file|table|json, required): Output data format.
- `description` (string, required): Description of the output content.
- `retention_policy` (string, optional): How long to keep this output.
- `sensitive` (boolean, optional): Whether output contains sensitive data. Default: false.
- `format_specification` (string, optional): Detailed format description for complex outputs.

## Relationships
- `CLI Utility Profile` 1---* `MCP Tool Listing` (one CLI command can expose multiple MCP tools for different steps).
- `CLI Utility Profile` 1---* `Execution Assurance Record` (each validation run references a CLI command).
- `MCP Tool Listing` 1---* `Execution Assurance Record` (records optionally include MCP tool name to trace parity).

