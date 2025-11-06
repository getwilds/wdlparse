use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Basic metadata extraction that works even with severely malformed WDL files
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct BasicWdlMetadata {
    pub version: Option<String>,
    pub workflow_name: Option<String>,
    pub task_names: Vec<String>,
}

impl BasicWdlMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract basic metadata from WDL content using regex patterns
    /// This is a fallback method that works even when the syntax tree parsing fails
    pub fn extract_from_text(content: &str) -> Self {
        let mut metadata = BasicWdlMetadata::new();

        // Extract version
        if let Some(version) = Self::extract_version(content) {
            metadata.version = Some(version);
        }

        // Extract workflow name (should be only one)
        if let Some(workflow_name) = Self::extract_workflow_name(content) {
            metadata.workflow_name = Some(workflow_name);
        }

        // Extract task names (can be multiple)
        metadata.task_names = Self::extract_task_names(content);

        metadata
    }

    /// Extract version from WDL content
    fn extract_version(content: &str) -> Option<String> {
        let version_regex = Regex::new(r"(?m)^\s*version\s+([^\s\n]+)").ok()?;
        version_regex
            .captures(content)?
            .get(1)
            .map(|m| m.as_str().to_string())
    }

    /// Extract workflow name from WDL content
    fn extract_workflow_name(content: &str) -> Option<String> {
        let workflow_regex =
            Regex::new(r"(?m)^\s*workflow\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{").ok()?;
        workflow_regex
            .captures(content)?
            .get(1)
            .map(|m| m.as_str().to_string())
    }

    /// Extract all task names from WDL content
    fn extract_task_names(content: &str) -> Vec<String> {
        let task_regex = match Regex::new(r"(?m)^\s*task\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{") {
            Ok(regex) => regex,
            Err(_) => return Vec::new(),
        };

        let mut task_names = HashSet::new();

        for captures in task_regex.captures_iter(content) {
            if let Some(task_match) = captures.get(1) {
                task_names.insert(task_match.as_str().to_string());
            }
        }

        let mut names: Vec<String> = task_names.into_iter().collect();
        names.sort();
        names
    }
}
