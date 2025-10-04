"""
Loco-rs Python Bindings

This package provides Python bindings for the Loco-rs code generator.
It exposes three main functions:
- generate_model: Generate a Loco model with migrations
- generate_scaffold: Generate a full scaffold (model + controller + views)
- generate_controller_view: Generate a controller with views
"""

from ._loco_bindings import (
    generate_model,
    generate_scaffold,
    generate_controller_view,
    ValidationError,
    FileOperationError,
    ProjectError,
)

__all__ = [
    "generate_model",
    "generate_scaffold", 
    "generate_controller_view",
    "ValidationError",
    "FileOperationError",
    "ProjectError",
]

__version__ = "0.1.0"

