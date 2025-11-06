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
        mermaid_wdl,
        mermaid_wdl_string,
        parse_wdl,
        parse_wdl_string,
    )
except ImportError as e:
    raise ImportError(
        "Failed to import the wdlparse Rust extension. "
        "Make sure the package was built correctly with maturin. "
        f"Original error: {e}"
    ) from e

__version__ = "0.1.1"
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
        self, file_path: str | Path, output_format: str = "human", extract_metadata: bool = False
    ) -> ParseResult:
        """
        Parse a WDL file from disk.

        Args:
            file_path: Path to the WDL file to parse
            output_format: Output format ("human", "json", or "tree")
            extract_metadata: Whether to extract basic metadata using robust fallback methods

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
        return parse_wdl(str(file_path), format_enum, self.verbose, extract_metadata)

    def parse_string(
        self, wdl_content: str, output_format: str = "human", extract_metadata: bool = False
    ) -> dict[str, Any]:
        """
        Parse WDL content from a string.

        Args:
            wdl_content: WDL source code as a string
            output_format: Output format ("human", "json", or "tree")
            extract_metadata: Whether to extract basic metadata using robust fallback methods

        Returns:
            Dictionary containing parse results and diagnostics

        Raises:
            ValueError: If the output format is invalid
        """
        format_enum = self._get_format_enum(output_format)
        return parse_wdl_string(wdl_content, format_enum, self.verbose, extract_metadata)

    def get_info(self, file_path: str | Path, output_format: str = "human", extract_metadata: bool = False) -> str:
        """
        Get information about a WDL file (version, tasks, workflows, etc.).

        Args:
            file_path: Path to the WDL file to analyze
            output_format: Output format ("human", "json", or "tree")
            extract_metadata: Whether to extract basic metadata using robust fallback methods

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
        return info_wdl(str(file_path), format_enum, extract_metadata)



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

    def mermaid(self, file_path: str | Path) -> str:
        """
        Generate a Mermaid diagram from a WDL file.

        Args:
            file_path: Path to the WDL file to generate diagram from

        Returns:
            String containing Mermaid diagram markup

        Raises:
            FileNotFoundError: If the file doesn't exist
        """
        file_path = Path(file_path)
        if not file_path.exists():
            raise FileNotFoundError(f"WDL file not found: {file_path}")

        return mermaid_wdl(str(file_path))

    def mermaid_string(self, wdl_content: str) -> str:
        """
        Generate a Mermaid diagram from WDL content string.

        Args:
            wdl_content: WDL source code as a string

        Returns:
            String containing Mermaid diagram markup
        """
        return mermaid_wdl_string(wdl_content)


# Convenience functions for direct use
def parse(
    file_path: str | Path, output_format: str = "human", verbose: bool = False, extract_metadata: bool = False
) -> ParseResult:
    """
    Parse a WDL file (convenience function).

    Args:
        file_path: Path to the WDL file to parse
        output_format: Output format ("human", "json", or "tree")
        verbose: Whether to include detailed diagnostic information
        extract_metadata: Whether to extract basic metadata using robust fallback methods

    Returns:
        ParseResult object containing parse results and diagnostics
    """
    parser = WDLParser(verbose=verbose)
    return parser.parse_file(file_path, output_format, extract_metadata)


def parse_text(
    wdl_content: str, output_format: str = "human", verbose: bool = False, extract_metadata: bool = False
) -> dict[str, Any]:
    """
    Parse WDL content from a string (convenience function).

    Args:
        wdl_content: WDL source code as a string
        output_format: Output format ("human", "json", or "tree")
        verbose: Whether to include detailed diagnostic information
        extract_metadata: Whether to extract basic metadata using robust fallback methods

    Returns:
        Dictionary containing parse results and diagnostics
    """
    parser = WDLParser(verbose=verbose)
    return parser.parse_string(wdl_content, output_format, extract_metadata)


def mermaid(file_path: str | Path) -> str:
    """
    Generate a Mermaid diagram from a WDL file (convenience function).

    Args:
        file_path: Path to the WDL file to generate diagram from

    Returns:
        String containing Mermaid diagram markup
    """
    parser = WDLParser()
    return parser.mermaid(file_path)


def mermaid_text(wdl_content: str) -> str:
    """
    Generate a Mermaid diagram from WDL content string (convenience function).

    Args:
        wdl_content: WDL source code as a string

    Returns:
        String containing Mermaid diagram markup
    """
    parser = WDLParser()
    return parser.mermaid_string(wdl_content)


def info(file_path: str | Path, output_format: str = "human", extract_metadata: bool = False) -> str:
    """
    Get information about a WDL file (convenience function).

    Args:
        file_path: Path to the WDL file to analyze
        output_format: Output format ("human", "json", or "tree")
        extract_metadata: Whether to extract basic metadata using robust fallback methods

    Returns:
        String containing file information
    """
    parser = WDLParser()
    return parser.get_info(file_path, output_format, extract_metadata)





# Make main exports available at package level
__all__ = [
    "WDLParser",
    "ParseResult",
    "OutputFormat",
    "parse",
    "parse_text",
    "info",
    "mermaid",
    "mermaid_text",
    "parse_wdl",
    "info_wdl",
    "parse_wdl_string",
    "mermaid_wdl",
    "mermaid_wdl_string",
    "PyOutputFormat",
]
