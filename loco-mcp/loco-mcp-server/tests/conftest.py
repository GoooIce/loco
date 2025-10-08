"""Test configuration for loco-mcp-server tests."""

import sys
from pathlib import Path


PACKAGE_ROOT = Path(__file__).resolve().parents[1]
SRC_PATH = PACKAGE_ROOT / "src"

if SRC_PATH.exists():
    sys.path.insert(0, str(SRC_PATH))


