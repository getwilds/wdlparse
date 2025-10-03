# wdlparse

[![Test](https://github.com/getwilds/wdlparse/actions/workflows/test.yml/badge.svg)](https://github.com/getwilds/wdlparse/actions/workflows/test.yml)

> [!NOTE]
> This is an alpha version.

A command-line tool for parsing WDL (Workflow Description Language) files.

## Installation

```bash
cargo install --path .
```

## Testing

To run tests, execute the following command:

```bash
cargo t
```

## The commands

```bash
wdlparse
```

```bash
A command-line tool for parsing WDL (Workflow Description Language) files

Usage: wdlparse <COMMAND>

Commands:
  parse    Parse a WDL file and display the syntax tree
  info     Show information about a WDL file (version, tasks, workflows, etc.)
  mermaid  Generate a Mermaid diagram from a WDL workflow
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help (see more with '--help')
  -V, --version  Print version
```

## Usage

### Parse a WDL file

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

### Get file information

```bash
# Show WDL file structure and metadata
wdlparse info examples/hello_world.wdl

# JSON output
wdlparse info examples/hello_world.wdl --format json
```

## Output Formats for parse and info

- **human**: User-friendly output with colors and formatting
- **json**: Machine-readable JSON output
- **tree**: Raw syntax tree output (parse command only)

### WDL to Mermaid

```bash
wdlparse mermaid examples/hello_world.wdl

# Convert to SVG with mmdc
npm install -g @mermaid-js/mermaid-cli
wdlparse mermaid examples/hello_world.wdl | mmdc -i - -o hello_world.svg
wdlparse mermaid examples/hello_world.wdl | mmdc -i - -o hello_world.png
```
