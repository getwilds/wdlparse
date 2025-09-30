# wdlparse

[![Test](https://github.com/getwilds/wdlparse/actions/workflows/test.yml/badge.svg)](https://github.com/getwilds/wdlparse/actions/workflows/test.yml)

> [!NOTE]
> This is an alpha version.

A command-line tool for parsing WDL (Workflow Description Language) files.

## Installation

### From source

```bash
cargo install --path .
```

### From releases

Go to the release pages to get the latest version.

<https://github.com/getwilds/wdlparse/releases>

```bash
cargo install --git https://github.com/getwilds/wdlparse --tag v0.0.5
```

## Testing

To run tests, execute the following command:

```bash
cargo t
```

## Usage

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

### Output Formats

- **human**: User-friendly output with colors and formatting
- **json**: Machine-readable JSON output
- **tree**: Raw syntax tree output (parse command only)
