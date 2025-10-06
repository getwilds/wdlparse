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


if __name__ == "__main__":
    # Allow running tests directly
    pytest.main([__file__, "-v"])
