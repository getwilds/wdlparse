#!/usr/bin/env python3
"""
Example demonstrating Mermaid diagram generation from WDL files using wdlparse.

This example shows how to:
1. Generate Mermaid diagrams from WDL files
2. Generate Mermaid diagrams from WDL content strings
3. Use both convenience functions and the WDLParser class
4. Save diagrams to files for use with Mermaid.js renderers
"""

from pathlib import Path

import wdlparse


def main():
    # Sample WDL workflow with multiple tasks and dependencies
    sample_wdl = """
version 1.1

task preprocess_data {
    input {
        File raw_data
        String sample_id
    }
    command {
        echo "Preprocessing ${sample_id}"
        process_data.py ${raw_data} > processed_${sample_id}.txt
    }
    output {
        File processed = "processed_${sample_id}.txt"
    }
}

task quality_check {
    input {
        File processed_data
    }
    command {
        quality_check.py ${processed_data} > qc_report.txt
    }
    output {
        File qc_report = "qc_report.txt"
        Boolean passed = read_boolean("qc_report.txt")
    }
}

task analyze_data {
    input {
        File processed_data
        String analysis_type = "standard"
    }
    command {
        analyze.py --type ${analysis_type} ${processed_data} > results.txt
    }
    output {
        File results = "results.txt"
    }
}

workflow data_pipeline {
    input {
        Array[File] input_files
        Array[String] sample_ids
        Boolean run_advanced_analysis = false
    }

    # Process each sample
    scatter (i in range(length(input_files))) {
        call preprocess_data {
            input:
                raw_data = input_files[i],
                sample_id = sample_ids[i]
        }

        call quality_check {
            input: processed_data = preprocess_data.processed
        }

        # Conditional analysis based on QC results
        if (quality_check.passed) {
            if (run_advanced_analysis) {
                call analyze_data as advanced_analysis {
                    input:
                        processed_data = preprocess_data.processed,
                        analysis_type = "advanced"
                }
            }

            if (!run_advanced_analysis) {
                call analyze_data as standard_analysis {
                    input:
                        processed_data = preprocess_data.processed,
                        analysis_type = "standard"
                }
            }
        }
    }

    output {
        Array[File] processed_files = preprocess_data.processed
        Array[File] qc_reports = quality_check.qc_report
        Array[File?] analysis_results = flatten([advanced_analysis.results, standard_analysis.results])
    }
}
"""

    print("=== Mermaid Diagram Generation Examples ===\n")

    # Example 1: Generate Mermaid diagram from string using convenience function
    print("1. Generating Mermaid diagram from WDL string (convenience function):")
    mermaid_diagram = wdlparse.mermaid_text(sample_wdl)
    print(mermaid_diagram)
    print("\n" + "="*70 + "\n")

    # Example 2: Using WDLParser class
    print("2. Using WDLParser class:")
    parser = wdlparse.WDLParser()
    mermaid_diagram = parser.mermaid_string(sample_wdl)

    # Save to file for use with Mermaid.js
    output_path = Path("workflow_diagram.mmd")
    with open(output_path, "w") as f:
        f.write(mermaid_diagram)

    print(f"Mermaid diagram saved to: {output_path}")
    print(f"Diagram contains {len(mermaid_diagram)} characters")
    print("\n" + "="*70 + "\n")

    # Example 3: Generate from existing WDL file
    print("3. Generating from existing WDL file:")
    example_file = Path("../examples/hello_world.wdl")

    if example_file.exists():
        try:
            # Using convenience function
            file_diagram = wdlparse.mermaid(example_file)
            print("Successfully generated diagram from file:")
            print(file_diagram[:300] + "..." if len(file_diagram) > 300 else file_diagram)

            # Using WDLParser class with Path object
            parser_diagram = parser.mermaid(example_file)
            assert parser_diagram == file_diagram, "Diagrams should be identical"
            print("✓ Both convenience function and WDLParser class produce identical results")

        except Exception as e:
            print(f"Error processing file: {e}")
    else:
        print(f"Example file not found: {example_file}")

    print("\n" + "="*70 + "\n")

    # Example 4: Simple workflow demonstration
    print("4. Simple workflow example:")
    simple_wdl = """
version 1.0

task hello {
    input {
        String name
    }
    command {
        echo "Hello ${name}!"
    }
    output {
        String greeting = stdout()
    }
}

workflow greet {
    input {
        String person = "World"
    }

    call hello { input: name = person }

    output {
        String message = hello.greeting
    }
}
"""

    simple_diagram = wdlparse.mermaid_text(simple_wdl)
    print("Simple workflow Mermaid diagram:")
    print(simple_diagram)

    print("\nℹ️  Usage Tips:")
    print("- Copy the generated Mermaid markup to any Mermaid.js renderer")
    print("- Use online tools like mermaid.live or GitHub's Mermaid support")
    print("- Save diagrams with .mmd extension for easy identification")
    print("- The diagrams include color-coded styling for different node types")


if __name__ == "__main__":
    main()
