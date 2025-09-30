# WDL to Mermaid Diagram Examples

This directory contains example Mermaid diagrams generated from WDL workflow files.

## Usage

Generate a Mermaid diagram from a WDL file:

```bash
# Print to stdout
wdlparse mermaid workflow.wdl

# Save to file
wdlparse mermaid workflow.wdl -o diagram.mmd
```

## Viewing Diagrams

You can view the generated Mermaid diagrams using:

1. **Online Mermaid Editor**: Copy the diagram code to [mermaid.live](https://mermaid.live)
2. **VS Code**: Install the "Mermaid Markdown Syntax Highlighting" extension
3. **GitHub**: Mermaid diagrams are natively supported in markdown files
4. **Command line**: Use `mmdc` (mermaid-cli) to render to PNG/SVG

## Examples

### Simple Workflow (`hello_world.mmd`)

Generated from `examples/hello_world.wdl`. Shows:
- A workflow with inputs and outputs
- A task definition
- A call statement connecting them

### Complex Pipeline (`complex_example.mmd`)

Generated from `examples/complex_example.wdl`. Shows:
- Multiple tasks (`align_reads`, `call_variants`)
- A workflow with multiple inputs/outputs
- Scatter operations (parallel execution)
- Conditional execution
- Task dependencies
- External tool calls

## Diagram Elements

The generated diagrams use different shapes and colors for different WDL elements:

| Element | Shape | Color | Description |
|---------|-------|-------|-------------|
| Workflow | `([name])` | Light green | Main workflow container |
| Task | `[name]` | Light blue | Task definitions |
| Call | `[call name]` | Light purple | Task calls within workflow |
| Input | `((Input: name))` | Green | Workflow/task inputs |
| Output | `((Output: name))` | Orange | Workflow/task outputs |
| Conditional | `{/if condition/}` | Yellow | If/conditional blocks |
| Scatter | `[/scatter var\]` | Pink | Scatter/parallel blocks |

## Arrows

- `-->` : Standard workflow connections
- `---|label|` : Dependency relationships between tasks

## Tips

- Use the `-o` flag to save diagrams to `.mmd` files for easier sharing
- The diagrams work best when viewed in a Mermaid renderer
- Complex workflows may generate large diagrams - consider breaking them into smaller sub-workflows
- Dependencies are extracted from variable references in call inputs (e.g., `task1.output`)