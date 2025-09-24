version 1.1

# This is a simple Hello World workflow in WDL

task say_hello {
    input {
        String name = "World"
        Int repetitions = 1
    }

    command <<<
        for i in $(seq 1 ~{repetitions}); do
            echo "Hello, ~{name}!"
        done
    >>>

    output {
        Array[String] greetings = read_lines(stdout())
    }

    runtime {
        docker: "ubuntu:20.04"
        memory: "1GB"
        cpu: 1
    }
}

workflow hello_world {
    input {
        String greeting_name
        Int times = 3
    }

    call say_hello {
        input:
            name = greeting_name,
            repetitions = times
    }

    output {
        Array[String] all_greetings = say_hello.greetings
    }

    meta {
        author: "WDL Parser Example"
        email: "example@example.com"
        description: "A simple hello world workflow"
    }
}
