#!/usr/bin/env python3
"""
Unit tests for the wdlparse Python library.
"""

import os
import tempfile
from pathlib import Path

import pytest
import wdlparse


class TestWDLParse:
    """Test cases for the wdlparse Python bindings."""

    @pytest.fixture
    def sample_wdl(self):
        """Sample WDL content for testing."""
        return """
version 1.0

task hello {
    input {
        String name = "World"
        Int count = 1
    }

    command {
        echo "Hello ${name}!" > output.txt
    }

    output {
        File greeting = "output.txt"
    }

    runtime {
        docker: "ubuntu:latest"
        memory: "1G"
    }
}

workflow hello_workflow {
    input {
        String person = "Alice"
    }

    call hello { input: name = person }

    output {
        File result = hello.greeting
    }
}
"""

    @pytest.fixture
    def invalid_wdl(self):
        """Invalid WDL content for error testing."""
        return """
version 1.0

task broken_task {
    input {
        String name = "test"
    # Missing closing brace

    command {
        echo "This has syntax errors"
    }
"""

    def test_parse_text_human_format(self, sample_wdl):
        """Test parsing WDL text with human format."""
        result = wdlparse.parse_text(sample_wdl, output_format="human")

        assert isinstance(result, dict)
        assert "output" in result
        assert "diagnostics_count" in result
        assert "has_errors" in result
        assert result["diagnostics_count"] >= 0
        assert isinstance(result["has_errors"], bool)

    def test_parse_text_json_format(self, sample_wdl):
        """Test parsing WDL text with JSON format."""
        result = wdlparse.parse_text(sample_wdl, output_format="json")

        assert isinstance(result, dict)
        assert "output" in result
        assert result["diagnostics_count"] >= 0

    def test_parse_text_tree_format(self, sample_wdl):
        """Test parsing WDL text with tree format."""
        result = wdlparse.parse_text(sample_wdl, output_format="tree")

        assert isinstance(result, dict)
        assert "output" in result
        assert result["diagnostics_count"] >= 0

    def test_parse_text_with_verbose(self, sample_wdl):
        """Test parsing WDL text with verbose output."""
        result = wdlparse.parse_text(sample_wdl, output_format="human", verbose=True)

        assert isinstance(result, dict)
        assert "output" in result
        assert "diagnostics_count" in result

    def test_parse_text_invalid_format(self, sample_wdl):
        """Test parsing with invalid output format."""
        with pytest.raises(ValueError, match="Invalid output format"):
            wdlparse.WDLParser().parse_string(sample_wdl, "invalid_format")

    def test_parse_file(self, sample_wdl):
        """Test parsing WDL from file."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".wdl", delete=False) as f:
            f.write(sample_wdl)
            temp_path = f.name

        try:
            result = wdlparse.parse(temp_path, output_format="human")

            assert isinstance(result, wdlparse.ParseResult)
            assert result.file_path == temp_path
            assert result.diagnostics_count >= 0
            assert isinstance(result.has_errors, bool)
            assert isinstance(result.output, str)

        finally:
            os.unlink(temp_path)

    def test_parse_nonexistent_file(self):
        """Test parsing non-existent file raises error."""
        with pytest.raises(FileNotFoundError):
            wdlparse.parse("/nonexistent/file.wdl")

    def test_info_file(self, sample_wdl):
        """Test getting info from WDL file."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".wdl", delete=False) as f:
            f.write(sample_wdl)
            temp_path = f.name

        try:
            result = wdlparse.info(temp_path, output_format="human")

            assert isinstance(result, str)
            assert len(result) > 0

        finally:
            os.unlink(temp_path)

    def test_info_json_format(self, sample_wdl):
        """Test getting info in JSON format."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".wdl", delete=False) as f:
            f.write(sample_wdl)
            temp_path = f.name

        try:
            result = wdlparse.info(temp_path, output_format="json")

            assert isinstance(result, str)
            assert len(result) > 0
            # Should be valid JSON
            import json

            json.loads(result)

        finally:
            os.unlink(temp_path)

    def test_wdl_parser_class(self, sample_wdl):
        """Test using the WDLParser class directly."""
        parser = wdlparse.WDLParser(verbose=True)

        result = parser.parse_string(sample_wdl, "human")
        assert isinstance(result, dict)
        assert "output" in result

        # Test with file
        with tempfile.NamedTemporaryFile(mode="w", suffix=".wdl", delete=False) as f:
            f.write(sample_wdl)
            temp_path = f.name

        try:
            result = parser.parse_file(temp_path, "human")
            assert isinstance(result, wdlparse.ParseResult)

            info_result = parser.get_info(temp_path, "human")
            assert isinstance(info_result, str)

        finally:
            os.unlink(temp_path)

    def test_parse_result_repr(self, sample_wdl):
        """Test ParseResult string representation."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".wdl", delete=False) as f:
            f.write(sample_wdl)
            temp_path = f.name

        try:
            result = wdlparse.parse(temp_path)
            repr_str = repr(result)

            assert "ParseResult" in repr_str
            assert temp_path in repr_str
            assert str(result.diagnostics_count) in repr_str

        finally:
            os.unlink(temp_path)

    def test_error_handling_invalid_wdl(self, invalid_wdl):
        """Test parsing invalid WDL handles errors gracefully."""
        result = wdlparse.parse_text(invalid_wdl, output_format="human", verbose=True)

        # Should not raise an exception, but should report diagnostics
        assert isinstance(result, dict)
        assert result["diagnostics_count"] > 0
        # Note: Whether it has_errors depends on the parser's strictness

    def test_output_format_enum(self):
        """Test OutputFormat enum values."""
        assert hasattr(wdlparse, "OutputFormat")
        assert hasattr(wdlparse.OutputFormat, "Human")
        assert hasattr(wdlparse.OutputFormat, "Json")
        assert hasattr(wdlparse.OutputFormat, "Tree")

    def test_module_exports(self):
        """Test that expected symbols are exported."""
        expected_exports = [
            "WDLParser",
            "ParseResult",
            "parse",
            "parse_text",
            "info",
            "parse_wdl",
            "info_wdl",
            "parse_wdl_string",
        ]

        for export in expected_exports:
            assert hasattr(wdlparse, export), f"Missing export: {export}"

    def test_version_attribute(self):
        """Test that version is available."""
        assert hasattr(wdlparse, "__version__")
        assert isinstance(wdlparse.__version__, str)

    def test_path_object_support(self, sample_wdl):
        """Test that Path objects are supported."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".wdl", delete=False) as f:
            f.write(sample_wdl)
            temp_path = Path(f.name)

        try:
            # Test with Path object
            parser = wdlparse.WDLParser()
            result = parser.parse_file(temp_path)
            assert isinstance(result, wdlparse.ParseResult)

            info_result = parser.get_info(temp_path)
            assert isinstance(info_result, str)

        finally:
            temp_path.unlink()


class TestRobustMetadata:
    """Test cases for robust WDL metadata extraction using extract_metadata parameter."""

    @pytest.fixture
    def malformed_wdl(self):
        """Malformed WDL content for testing robust extraction."""
        return """
version 1.1

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

    output {
        String result1 = broken_task.result
    }

    # Missing closing brace for workflow
"""

    @pytest.fixture
    def multi_task_wdl(self):
        """WDL with multiple tasks for testing."""
        return """
version 1.0

task task_alpha {
    input {
        String input1
    }
    command { echo "${input1}" }
    output {
        String output1 = stdout()
    }
}

task task_beta {
    input {
        String input2
    }
    command { echo "${input2}" }
    output {
        String output2 = stdout()
    }
}

workflow multi_task_workflow {
    call task_alpha
    call task_beta
}
"""

    def test_parse_text_with_extract_metadata_valid(self, multi_task_wdl):
        """Test parse_text with extract_metadata=True for valid WDL."""
        result = wdlparse.parse_text(multi_task_wdl, output_format="json", extract_metadata=True)

        assert isinstance(result, dict)
        assert "basic_metadata" in result

        # Extract the basic_metadata tuple (version, workflow_name, task_names)
        basic_metadata = result["basic_metadata"]
        assert basic_metadata[0] == "1.0"  # version
        assert basic_metadata[1] == "multi_task_workflow"  # workflow_name
        assert set(basic_metadata[2]) == {"task_alpha", "task_beta"}  # task_names

    def test_parse_text_with_extract_metadata_malformed(self, malformed_wdl):
        """Test parse_text with extract_metadata=True for malformed WDL."""
        result = wdlparse.parse_text(malformed_wdl, output_format="json", extract_metadata=True)

        assert isinstance(result, dict)
        assert "basic_metadata" in result
        assert result["diagnostics_count"] > 0

        basic_metadata = result["basic_metadata"]
        assert basic_metadata[0] == "1.1"  # version
        assert basic_metadata[1] == "broken_workflow"  # workflow_name
        assert basic_metadata[2] == ["broken_task"]  # task_names

    def test_parse_file_with_extract_metadata(self, multi_task_wdl):
        """Test parse file with extract_metadata=True."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".wdl", delete=False) as f:
            f.write(multi_task_wdl)
            temp_path = f.name

        try:
            result = wdlparse.parse(temp_path, output_format="json", extract_metadata=True)

            assert isinstance(result, wdlparse.ParseResult)
            # For ParseResult, the basic_metadata would be in the JSON output
            import json
            output_data = json.loads(result.output)
            assert "basic_metadata" in output_data

        finally:
            os.unlink(temp_path)

    def test_info_with_extract_metadata(self, multi_task_wdl):
        """Test info command with extract_metadata=True."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".wdl", delete=False) as f:
            f.write(multi_task_wdl)
            temp_path = f.name

        try:
            result = wdlparse.info(temp_path, output_format="json", extract_metadata=True)

            # Parse the JSON result
            import json
            info_data = json.loads(result)
            assert "basic_metadata" in info_data

            basic_metadata = info_data["basic_metadata"]
            assert basic_metadata["version"] == "1.0"
            assert basic_metadata["workflow_name"] == "multi_task_workflow"
            assert set(basic_metadata["task_names"]) == {"task_alpha", "task_beta"}

        finally:
            os.unlink(temp_path)

    def test_extract_metadata_false_by_default(self, multi_task_wdl):
        """Test that extract_metadata=False by default (no extra work)."""
        result = wdlparse.parse_text(multi_task_wdl, output_format="json")

        assert isinstance(result, dict)
        assert "basic_metadata" not in result  # Should not be included by default

    def test_extract_metadata_no_workflow(self):
        """Test extract_metadata with tasks but no workflow."""
        wdl_content = """
version 1.0

task standalone_task {
    input {
        String message
    }
    command {
        echo "${message}"
    }
    output {
        String result = stdout()
    }
}

task another_task {
    input {
        Int number
    }
    command {
        echo "Number: ${number}"
    }
    output {
        String result = stdout()
    }
}
"""
        result = wdlparse.parse_text(wdl_content, output_format="json", extract_metadata=True)

        basic_metadata = result["basic_metadata"]
        assert basic_metadata[0] == "1.0"  # version
        assert basic_metadata[1] is None  # workflow_name
        assert set(basic_metadata[2]) == {"standalone_task", "another_task"}  # task_names

    def test_extract_metadata_no_version(self):
        """Test extract_metadata when version is missing."""
        wdl_content = """
task simple_task {
    input {
        String input_val
    }
    command {
        echo "${input_val}"
    }
    output {
        String output_val = stdout()
    }
}

workflow simple_workflow {
    call simple_task
}
"""
        result = wdlparse.parse_text(wdl_content, output_format="json", extract_metadata=True)

        basic_metadata = result["basic_metadata"]
        assert basic_metadata[0] is None  # version
        assert basic_metadata[1] == "simple_workflow"  # workflow_name
        assert basic_metadata[2] == ["simple_task"]  # task_names

    def test_wdl_parser_class_with_extract_metadata(self, multi_task_wdl):
        """Test WDLParser class methods with extract_metadata parameter."""
        parser = wdlparse.WDLParser()

        # Test parse_string with extract_metadata
        result = parser.parse_string(multi_task_wdl, "json", extract_metadata=True)
        assert "basic_metadata" in result

        # Test with file
        with tempfile.NamedTemporaryFile(mode="w", suffix=".wdl", delete=False) as f:
            f.write(multi_task_wdl)
            temp_path = f.name

        try:
            # Test parse_file with extract_metadata
            result = parser.parse_file(temp_path, "json", extract_metadata=True)
            import json
            output_data = json.loads(result.output)
            assert "basic_metadata" in output_data

            # Test get_info with extract_metadata
            info_result = parser.get_info(temp_path, "json", extract_metadata=True)
            import json
            info_data = json.loads(info_result)
            assert "basic_metadata" in info_data

        finally:
            os.unlink(temp_path)

    def test_extract_metadata_compared_to_regular_parsing(self, malformed_wdl):
        """Test that extract_metadata provides additional info when regular parsing has errors."""
        # Regular parsing without extract_metadata
        regular_result = wdlparse.parse_text(malformed_wdl, output_format="json", verbose=False)

        # Should have diagnostics/errors but no basic_metadata
        assert regular_result["diagnostics_count"] > 0
        assert "basic_metadata" not in regular_result

        # With extract_metadata, should get additional robust extraction
        enhanced_result = wdlparse.parse_text(malformed_wdl, output_format="json", extract_metadata=True)
        assert enhanced_result["diagnostics_count"] > 0
        assert "basic_metadata" in enhanced_result

        basic_metadata = enhanced_result["basic_metadata"]
        assert basic_metadata[0] == "1.1"  # version
        assert basic_metadata[1] == "broken_workflow"  # workflow_name
        assert basic_metadata[2] == ["broken_task"]  # task_names


if __name__ == "__main__":
    # Allow running tests directly
    pytest.main([__file__, "-v"])
