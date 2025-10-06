# wdlparse

[![Test](https://github.com/getwilds/wdlparse/actions/workflows/test.yml/badge.svg)](https://github.com/getwilds/wdlparse/actions/workflows/test.yml)

> [!NOTE]
> This is an alpha version.

A command-line tool and Python library for parsing WDL (Workflow Description Language) files.

wdlparse provides both a CLI tool and Python bindings for parsing and analyzing WDL files with high performance and detailed diagnostics.

## Installation

### CLI Tool

If you don't have Rust/cargo installed, go to <https://rustup.rs/> to get them installed.

#### From source

```bash
cargo install --path . --bin wdlparse
```

#### From releases

Go to the release pages to get the latest version.

<https://github.com/getwilds/wdlparse/releases>

```bash
cargo install --git https://github.com/getwilds/wdlparse --tag v0.0.5 --bin wdlparse
```

### Python Library

Install using `uv` (recommended) or `pip`:

```bash
# Clone the repository
git clone https://github.com/getwilds/wdlparse
cd wdlparse

# Install with uv
uv add --dev maturin
uv run maturin develop --release

# Or with pip in a virtual environment
pip install maturin
maturin develop --release
```

## Testing

### Rust Tests
```bash
cargo test
```

### Python Tests
```bash
# With uv
uv run pytest python/tests/ -v

# Or with pip
pytest python/tests/ -v
```

## CLI Usage

### Basic Commands

#### Parse a WDL file

```bash
# Display syntax tree (default)
wdlparse parse examples/hello_world.wdl

# Human-readable output
wdlparse parse examples/hello_world.wdl --format human

# JSON output
wdlparse parse examples/hello_world.wdl --format json

# Verbose output with diagnostics
wdlparse parse examples/hello_world.wdl --verbose
```

#### Get file information

```bash
# Show WDL file structure and metadata
wdlparse info examples/hello_world.wdl

# JSON output
wdlparse info examples/hello_world.wdl --format json
```

### CLI Output Formats

- **human**: User-friendly output with colors and formatting
- **json**: Machine-readable JSON output
- **tree**: Raw syntax tree output (parse command only)

## Python Library

wdlparse provides Python bindings built with PyO3 and maturin for high-performance WDL parsing directly from Python.

### Python Usage

```python
import wdlparse

# Parse WDL from string
wdl_content = """
version 1.0

task hello {
    input {
        String name = "World"
    }
    
    command {
        echo "Hello ${name}!"
    }
    
    output {
        String greeting = stdout()
    }
}
"""

# Parse with different output formats
result = wdlparse.parse_text(wdl_content, output_format="human", verbose=True)
print(f"Diagnostics: {result['diagnostics_count']}")
print(f"Has errors: {result['has_errors']}")
print(result['output'])

# Parse from file
result = wdlparse.parse("path/to/file.wdl", output_format="json")
print(f"File: {result.file_path}")
print(result.output)

# Get file information
info = wdlparse.info("path/to/file.wdl", output_format="human")
print(info)

# Using the high-level API
parser = wdlparse.WDLParser(verbose=True)
result = parser.parse_string(wdl_content)
```

### Python API Reference

#### Functions

- `parse_text(content, output_format="human", verbose=False)` - Parse WDL from string
- `parse(file_path, output_format="human", verbose=False)` - Parse WDL from file
- `info(file_path, output_format="human")` - Get WDL file information

#### Classes

- `WDLParser(verbose=False)` - High-level parser interface
- `ParseResult` - Contains parsing results and diagnostics
- `OutputFormat` - Enum for output format options (Human, Json, Tree)

#### Python Output Formats

- `"human"` - User-friendly formatted output
- `"json"` - Structured JSON output
- `"tree"` - Syntax tree representation

### Python Examples and Tests

```bash
# Run the example (with uv)
uv run python python/examples/usage.py

# Run tests (with uv)  
uv run pytest python/tests/ -v
```

## Development

For detailed build instructions and development setup, see [BUILD.md](BUILD.md).

### Building Both CLI and Python Library

This repository supports building both the CLI tool and Python library from the same codebase using Cargo features:

- **CLI only**: `cargo build --bin wdlparse` (default, no Python dependencies)
- **Python library**: `maturin develop` (enables `python` feature with PyO3)

The Python bindings are conditionally compiled using the `python` feature flag, allowing the CLI to be built without any Python dependencies while still supporting the full Python interface when needed.
