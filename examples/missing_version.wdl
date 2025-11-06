workflow oops {
  call oopsie
}

#### TASK DEFINITIONS

task oopsie {
  input {
    String str
  }
  command { echo ${str} }
  runtime { docker: docker_image }
}