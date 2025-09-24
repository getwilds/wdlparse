# wdlparse

> [!WARNING]
> BEWARE - This was written by Claude - with some edits by me.

A command-line tool for parsing WDL (Workflow Description Language) files.

## Installation

```bash
cargo install --path .
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
