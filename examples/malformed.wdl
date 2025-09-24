version 1.1

# This is a malformed WDL file to test error handling

task broken_task {
    input {
        String name
        Int count = "not a number"  # Type mismatch error
        missing_type variable_name  # Missing type keyword
    }

    command <<<
        echo "Hello ~{name}"
        # Missing closing brace for placeholder
        echo "Count: ~{count"
    >>>

    output {
        String result = stdout()
        # Missing comma between outputs
        Int final_count = count
    }

    runtime {
        docker: "ubuntu:20.04"
        memory: 1GB  # Missing quotes around memory value
        cpu:
    }
}

workflow broken_workflow {
    input {
        String workflow_name
        Array[String] items
    }

    # Missing call keyword
    broken_task {
        input:
            name = workflow_name,
            count = length(items)
    }

    # Undefined variable reference
    call broken_task as second_call {
        input:
            name = undefined_variable,
            count = 5
    }

    output {
        String result1 = broken_task.result
        String result2 = second_call.result
    }

    # Missing closing brace for workflow
