#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyDict;
#[cfg(feature = "python")]
use serde_json;
#[cfg(feature = "python")]
use std::path::PathBuf;
#[cfg(feature = "python")]
use wdl_grammar::SyntaxTree;

pub mod commands;
pub mod info;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
    /// Syntax tree format
    Tree,
}

#[cfg(feature = "python")]
#[derive(Clone, Debug)]
#[pyclass]
pub enum PyOutputFormat {
    Human,
    Json,
    Tree,
}

#[cfg(feature = "python")]
impl From<PyOutputFormat> for OutputFormat {
    fn from(format: PyOutputFormat) -> Self {
        match format {
            PyOutputFormat::Human => OutputFormat::Human,
            PyOutputFormat::Json => OutputFormat::Json,
            PyOutputFormat::Tree => OutputFormat::Tree,
        }
    }
}

#[cfg(feature = "python")]
#[derive(Clone, Debug)]
#[pyclass]
pub struct ParseResult {
    #[pyo3(get)]
    pub file_path: String,
    #[pyo3(get)]
    pub diagnostics_count: usize,
    #[pyo3(get)]
    pub has_errors: bool,
    #[pyo3(get)]
    pub output: String,
}

#[cfg(feature = "python")]
#[pymethods]
impl ParseResult {
    fn __repr__(&self) -> String {
        format!(
            "ParseResult(file_path='{}', diagnostics_count={}, has_errors={}, output_length={})",
            self.file_path,
            self.diagnostics_count,
            self.has_errors,
            self.output.len()
        )
    }
}

/// Parse a WDL file and return structured results
#[cfg(feature = "python")]
#[pyfunction]
fn parse_wdl(
    file_path: String,
    format: Option<PyOutputFormat>,
    verbose: Option<bool>,
) -> PyResult<ParseResult> {
    let format = format.unwrap_or(PyOutputFormat::Human);
    let verbose = verbose.unwrap_or(false);
    let path = PathBuf::from(&file_path);

    // Read the file content
    let content = std::fs::read_to_string(&path).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
            "Failed to read file '{}': {}",
            file_path, e
        ))
    })?;

    // Parse the WDL content
    let (tree, diagnostics) = SyntaxTree::parse(&content);
    let has_errors = diagnostics
        .iter()
        .any(|d| matches!(d.severity(), wdl_grammar::Severity::Error));

    // Generate output based on format
    let output = match format {
        PyOutputFormat::Tree => {
            let mut result = String::new();
            if verbose && !diagnostics.is_empty() {
                result.push_str("Diagnostics:\n");
                for diagnostic in &diagnostics {
                    result.push_str(&format!(
                        "  {:?}: {}\n",
                        diagnostic.severity(),
                        diagnostic.message()
                    ));
                }
                result.push('\n');
            }
            result.push_str("Syntax Tree:\n");
            result.push_str(&format!("{:#?}", tree));
            result
        }
        PyOutputFormat::Json => {
            let semantic_info = commands::extract_semantic_info(&tree.root());
            let json_output = serde_json::json!({
                "file": file_path,
                "diagnostics": diagnostics.len(),
                "has_errors": has_errors,
                "wdl": semantic_info
            });
            serde_json::to_string_pretty(&json_output).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to serialize to JSON: {}",
                    e
                ))
            })?
        }
        PyOutputFormat::Human => {
            let mut result = String::new();
            result.push_str(&format!("Parsed: {}\n", file_path));
            result.push_str(&format!("Root node: {:?}\n", tree.root().kind()));
            if !diagnostics.is_empty() {
                result.push_str(&format!("Diagnostics: {}\n", diagnostics.len()));
                if verbose {
                    for diagnostic in &diagnostics {
                        result.push_str(&format!(
                            "  {:?}: {}\n",
                            diagnostic.severity(),
                            diagnostic.message()
                        ));
                    }
                }
            } else {
                result.push_str("No issues found\n");
            }
            result
        }
    };

    Ok(ParseResult {
        file_path,
        diagnostics_count: diagnostics.len(),
        has_errors,
        output,
    })
}

/// Get information about a WDL file (version, tasks, workflows, etc.)
#[cfg(feature = "python")]
#[pyfunction]
fn info_wdl(file_path: String, format: Option<PyOutputFormat>) -> PyResult<String> {
    let format = format.unwrap_or(PyOutputFormat::Human);
    let path = PathBuf::from(&file_path);

    let content = std::fs::read_to_string(&path).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
            "Failed to read file '{}': {}",
            file_path, e
        ))
    })?;

    let (tree, _) = SyntaxTree::parse(&content);
    let semantic_info = commands::extract_semantic_info(&tree.root());

    let result = match format {
        PyOutputFormat::Json => {
            let json_output = serde_json::json!({
                "file": file_path,
                "wdl": semantic_info
            });
            serde_json::to_string_pretty(&json_output).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to serialize to JSON: {}",
                    e
                ))
            })?
        }
        PyOutputFormat::Human => {
            let mut result = String::new();
            result.push_str(&format!("WDL File: {}\n", file_path));
            if let Some(version) = &semantic_info.version {
                result.push_str(&format!("Version: {}\n", version));
            }
            result.push_str(&format!("Tasks: {}\n", semantic_info.tasks.len()));
            result.push_str(&format!("Workflows: {}\n", semantic_info.workflows.len()));
            result.push_str(&format!("Structs: {}\n", semantic_info.structs.len()));
            result.push_str(&format!("Imports: {}\n", semantic_info.imports.len()));
            result
        }
        PyOutputFormat::Tree => {
            format!(
                "Semantic Info: tasks={}, workflows={}, structs={}, imports={}",
                semantic_info.tasks.len(),
                semantic_info.workflows.len(),
                semantic_info.structs.len(),
                semantic_info.imports.len()
            )
        }
    };

    Ok(result)
}

/// Parse WDL content from a string instead of a file
#[cfg(feature = "python")]
#[pyfunction]
fn parse_wdl_string(
    py: Python<'_>,
    content: String,
    format: Option<PyOutputFormat>,
    verbose: Option<bool>,
) -> PyResult<Py<PyDict>> {
    let format = format.unwrap_or(PyOutputFormat::Human);
    let verbose = verbose.unwrap_or(false);

    // Parse the WDL content
    let (tree, diagnostics) = SyntaxTree::parse(&content);
    let has_errors = diagnostics
        .iter()
        .any(|d| matches!(d.severity(), wdl_grammar::Severity::Error));

    let dict = PyDict::new(py);

    dict.set_item("diagnostics_count", diagnostics.len())?;
    dict.set_item("has_errors", has_errors)?;

    // Add diagnostic details if verbose
    if verbose {
        let diagnostic_list: Vec<(String, String)> = diagnostics
            .iter()
            .map(|d| (format!("{:?}", d.severity()), d.message().to_string()))
            .collect();
        dict.set_item("diagnostics", diagnostic_list)?;
    }

    // Generate output based on format
    let output = match format {
        PyOutputFormat::Tree => {
            format!("{:#?}", tree)
        }
        PyOutputFormat::Json => {
            let semantic_info = commands::extract_semantic_info(&tree.root());
            let json_output = serde_json::json!({
                "diagnostics": diagnostics.len(),
                "has_errors": has_errors,
                "wdl": semantic_info
            });
            serde_json::to_string_pretty(&json_output).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to serialize to JSON: {}",
                    e
                ))
            })?
        }
        PyOutputFormat::Human => {
            let mut result = String::new();
            result.push_str(&format!("Root node: {:?}\n", tree.root().kind()));
            if !diagnostics.is_empty() {
                result.push_str(&format!("Diagnostics: {}\n", diagnostics.len()));
                if verbose {
                    for diagnostic in &diagnostics {
                        result.push_str(&format!(
                            "  {:?}: {}\n",
                            diagnostic.severity(),
                            diagnostic.message()
                        ));
                    }
                }
            } else {
                result.push_str("No issues found\n");
            }
            result
        }
    };

    dict.set_item("output", output)?;
    Ok(dict.unbind())
}

/// A Python module implemented in Rust.
#[cfg(feature = "python")]
#[pymodule]
fn wdlparse(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyOutputFormat>()?;
    m.add_class::<ParseResult>()?;
    m.add_function(wrap_pyfunction!(parse_wdl, m)?)?;
    m.add_function(wrap_pyfunction!(info_wdl, m)?)?;
    m.add_function(wrap_pyfunction!(parse_wdl_string, m)?)?;
    Ok(())
}
