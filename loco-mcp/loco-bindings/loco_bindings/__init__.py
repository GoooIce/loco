"""
Loco-rs Python Bindings

This package provides Python bindings for the Loco-rs code generator.
It exposes three main functions:
- generate_model: Generate a Loco model with migrations
- generate_scaffold: Generate a full scaffold (model + controller + views)
- generate_controller_view: Generate a controller with views

CLI utility functions:
- migrate_db: Execute database migration
- rotate_keys: Rotate service account keys
- clean_temp: Clean temporary files
"""

from ._loco_bindings import (
    generate_model,
    generate_scaffold,
    generate_controller_view,
    migrate_db,
    rotate_keys,
    clean_temp,
    ValidationError,
    FileOperationError,
    ProjectError,
)

__all__ = [
    "generate_model",
    "generate_scaffold", 
    "generate_controller_view",
    "migrate_db",
    "rotate_keys",
    "clean_temp",
    "ValidationError",
    "FileOperationError",
    "ProjectError",
]

__version__ = "0.1.0"

