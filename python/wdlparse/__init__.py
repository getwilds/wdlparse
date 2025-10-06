"""
wdlparse: A Python interface to the wdlparse Rust library for parsing WDL files.

This package provides Python bindings for parsing and analyzing WDL (Workflow Description Language) files.
"""

from pathlib import Path
from typing import Any

# Import the Rust extension module
try:
    from .wdlparse import (
        ParseResult,
        PyOutputFormat,
        info_wdl,
        parse_wdl,
        parse_wdl_string,
    )
except ImportError as e:
    raise ImportError(
        "Failed to import the wdlparse Rust extension. "
        "Make sure the package was built correctly with maturin. "
        f"Original error: {e}"
    ) from e

__version__ = "0.1.0"
__author__ = "Scott Chamberlain"
__email__ = "sachamber@fredhutch.org"

# Re-export main classes and enums for convenience
OutputFormat = PyOutputFormat


class WDLParser:
    """High-level interface for parsing WDL files."""

    def __init__(self, verbose: bool = False):
        """
        Initialize the WDL parser.

        Args:
            verbose: Whether to include detailed diagnostic information
        """
        self.verbose = verbose

    def parse_file(
        self, file_path: str | Path, output_format: str = "human"
    ) -> ParseResult:
        """
        Parse a WDL file from disk.

        Args:
            file_path: Path to the WDL file to parse
            output_format: Output format ("human", "json", or "tree")

        Returns:
            ParseResult object containing parse results and diagnostics

        Raises:
            FileNotFoundError: If the file doesn't exist
            ValueError: If the output format is invalid
        """
        file_path = Path(file_path)
        if not file_path.exists():
            raise FileNotFoundError(f"WDL file not found: {file_path}")

        format_enum = self._get_format_enum(output_format)
        return parse_wdl(str(file_path), format_enum, self.verbose)

    def parse_string(
        self, wdl_content: str, output_format: str = "human"
    ) -> dict[str, Any]:
        """
        Parse WDL content from a string.

        Args:
            wdl_content: WDL source code as a string
            output_format: Output format ("human", "json", or "tree")

        Returns:
            Dictionary containing parse results and diagnostics

        Raises:
            ValueError: If the output format is invalid
        """
        format_enum = self._get_format_enum(output_format)
        return parse_wdl_string(wdl_content, format_enum, self.verbose)

    def get_info(self, file_path: str | Path, output_format: str = "human") -> str:
        """
        Get information about a WDL file (version, tasks, workflows, etc.).

        Args:
            file_path: Path to the WDL file to analyze
            output_format: Output format ("human", "json", or "tree")

        Returns:
            String containing file information

        Raises:
            FileNotFoundError: If the file doesn't exist
            ValueError: If the output format is invalid
        """
        file_path = Path(file_path)
        if not file_path.exists():
            raise FileNotFoundError(f"WDL file not found: {file_path}")

        format_enum = self._get_format_enum(output_format)
        return info_wdl(str(file_path), format_enum)

    def _get_format_enum(self, format_str: str) -> PyOutputFormat:
        """Convert string format to enum."""
        format_map = {
            "human": PyOutputFormat.Human,
            "json": PyOutputFormat.Json,
            "tree": PyOutputFormat.Tree,
        }

        if format_str.lower() not in format_map:
            raise ValueError(
                f"Invalid output format: {format_str}. "
                f"Valid options are: {list(format_map.keys())}"
            )

        return format_map[format_str.lower()]


# Convenience functions for direct use
def parse(
    file_path: str | Path, output_format: str = "human", verbose: bool = False
) -> ParseResult:
    """
    Parse a WDL file (convenience function).

    Args:
        file_path: Path to the WDL file to parse
        output_format: Output format ("human", "json", or "tree")
        verbose: Whether to include detailed diagnostic information

    Returns:
        ParseResult object containing parse results and diagnostics
    """
    parser = WDLParser(verbose=verbose)
    return parser.parse_file(file_path, output_format)


def parse_text(
    wdl_content: str, output_format: str = "human", verbose: bool = False
) -> dict[str, Any]:
    """
    Parse WDL content from a string (convenience function).

    Args:
        wdl_content: WDL source code as a string
        output_format: Output format ("human", "json", or "tree")
        verbose: Whether to include detailed diagnostic information

    Returns:
        Dictionary containing parse results and diagnostics
    """
    parser = WDLParser(verbose=verbose)
    return parser.parse_string(wdl_content, output_format)


def info(file_path: str | Path, output_format: str = "human") -> str:
    """
    Get information about a WDL file (convenience function).

    Args:
        file_path: Path to the WDL file to analyze
        output_format: Output format ("human", "json", or "tree")

    Returns:
        String containing file information
    """
    parser = WDLParser()
    return parser.get_info(file_path, output_format)


# Make main exports available at package level
__all__ = [
    "WDLParser",
    "ParseResult",
    "OutputFormat",
    "parse",
    "parse_text",
    "info",
    "parse_wdl",
    "info_wdl",
    "parse_wdl_string",
    "PyOutputFormat",
]
