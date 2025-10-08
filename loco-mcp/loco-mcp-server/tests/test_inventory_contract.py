"""Contract tests for CLI utilities inventory manifest."""

import pathlib

import yaml

INVENTORY_PATH = pathlib.Path(
    "/Users/devel0per/Code/framework/loco/specs/002-loco-bindings-mcp/contracts/cli-utilities.inventory.yaml"
)


def load_inventory() -> list[dict]:
    data = yaml.safe_load(INVENTORY_PATH.read_text())
    assert "cli_utilities" in data, "Manifest must define cli_utilities root key"
    return data["cli_utilities"]


def test_inventory_entries_include_guardrails_metadata() -> None:
    """Every CLI utility must expose ownership, approvals, timeout, and dependencies."""

    utilities = load_inventory()
    assert utilities, "Inventory must list at least one CLI utility"

    failures = {}

    for entry in utilities:
        missing = [
            field
            for field in ("owner", "approvals", "timeout", "dependencies")
            if not entry.get(field)
        ]

        if missing:
            failures[entry.get("id", "<unknown>")] = missing

    assert not failures, f"Manifest missing guardrail data: {failures}"

