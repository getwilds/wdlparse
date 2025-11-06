use crate::info::{
    CallInfo, CallInputItem, ImportInfo, InputInfo, MetaItem, OutputInfo, RuntimeItem, StructInfo,
    TaskInfo, WdlInfo, WorkflowInfo,
};
use crate::mermaid::{extract_workflow_graph, generate_mermaid};
use crate::metadata::BasicWdlMetadata;
use crate::OutputFormat;
use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::PathBuf;
use wdl_grammar::{SyntaxKind, SyntaxTree};

pub fn parse_command(
    file: PathBuf,
    format: OutputFormat,
    verbose: bool,
    extract_metadata: bool,
) -> Result<()> {
    let content = read_wdl_file(&file)?;
    let (tree, diagnostics) = SyntaxTree::parse(&content);

    // Extract basic metadata if requested
    let basic_metadata = if extract_metadata {
        Some(BasicWdlMetadata::extract_from_text(&content))
    } else {
        None
    };

    if verbose && !diagnostics.is_empty() {
        println!("{}", "Diagnostics:".yellow().bold());
        for diagnostic in &diagnostics {
            println!(
                "  {}: {}",
                format!("{:?}", diagnostic.severity()).red(),
                diagnostic.message()
            );
        }
        println!();
    }

    match format {
        OutputFormat::Tree => {
            println!("{}", "Syntax Tree:".green().bold());
            println!("{:#?}", tree);
        }
        OutputFormat::Json => {
            let semantic_info = extract_semantic_info(&tree.root());
            let mut json_output = serde_json::json!({
                "file": file.display().to_string(),
                "diagnostics": diagnostics.len(),
                "has_errors": diagnostics.iter().any(|d| matches!(d.severity(), wdl_grammar::Severity::Error)),
                "wdl": semantic_info
            });

            if let Some(metadata) = &basic_metadata {
                json_output["basic_metadata"] = serde_json::to_value(metadata)?;
            }

            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        OutputFormat::Human => {
            println!("{} {}", "Parsed:".green().bold(), file.display());
            println!("Root node: {}", format!("{:?}", tree.root().kind()).cyan());
            if !diagnostics.is_empty() {
                println!("Diagnostics: {}", diagnostics.len().to_string().yellow());
            } else {
                println!("{}", "No issues found".green());
            }
        }
    }

    Ok(())
}

pub fn info_command(file: PathBuf, format: OutputFormat, extract_metadata: bool) -> Result<()> {
    let content = read_wdl_file(&file)?;
    let (tree, diagnostics) = SyntaxTree::parse(&content);

    let mut info = WdlInfo::new();
    collect_semantic_info(&tree.root(), &mut info);

    // Extract basic metadata if requested
    let basic_metadata = if extract_metadata {
        Some(BasicWdlMetadata::extract_from_text(&content))
    } else {
        None
    };

    match format {
        OutputFormat::Json => {
            let mut json_output = serde_json::json!({
                "file": file.display().to_string(),
                "version": info.version,
                "tasks": info.tasks,
                "workflows": info.workflows,
                "structs": info.structs,
                "imports": info.imports
            });

            if let Some(metadata) = &basic_metadata {
                json_output["basic_metadata"] = serde_json::to_value(metadata)?;
            }

            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        _ => {
            println!("{} {}", "WDL File Info:".cyan().bold(), file.display());
            println!("{}", "─".repeat(50));

            if let Some(version) = &info.version {
                println!("{}: {}", "Version".green().bold(), version);
            }

            println!("{}: {}", "Tasks".green().bold(), info.tasks.len());
            for task in &info.tasks {
                println!("  • {}", task.name);
            }

            println!("{}: {}", "Workflows".green().bold(), info.workflows.len());
            for workflow in &info.workflows {
                println!("  • {}", workflow.name);
            }

            println!("{}: {}", "Structs".green().bold(), info.structs.len());
            for struct_name in &info.structs {
                println!("  • {}", struct_name.name);
            }

            println!("{}: {}", "Imports".green().bold(), info.imports.len());
            for import in &info.imports {
                let display = if let Some(alias) = &import.alias {
                    format!("{} as {}", import.uri, alias)
                } else {
                    import.uri.clone()
                };
                println!("  • {}", display);
            }

            if !diagnostics.is_empty() {
                println!();
                println!("{}: {}", "Diagnostics".yellow().bold(), diagnostics.len());
            }
        }
    }

    Ok(())
}

pub fn mermaid_command(file: PathBuf, output: Option<PathBuf>) -> Result<()> {
    let content = read_wdl_file(&file)?;

    let graph = extract_workflow_graph(&content)
        .map_err(|e| anyhow::anyhow!("Failed to extract workflow graph: {}", e))?;

    let mermaid_diagram = generate_mermaid(&graph);

    match output {
        Some(output_path) => {
            fs::write(&output_path, &mermaid_diagram)
                .with_context(|| format!("Failed to write to file: {}", output_path.display()))?;
            println!(
                "{} Mermaid diagram written to: {}",
                "Success:".green().bold(),
                output_path.display()
            );
        }
        None => {
            println!("{}", mermaid_diagram);
        }
    }

    Ok(())
}

fn read_wdl_file(path: &PathBuf) -> Result<String> {
    if !path.exists() {
        anyhow::bail!("File does not exist: {}", path.display());
    }

    if let Some(extension) = path.extension() {
        if extension != "wdl" {
            eprintln!(
                "{} File does not have .wdl extension: {}",
                "Warning:".yellow().bold(),
                path.display()
            );
        }
    }

    fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path.display()))
}

pub fn extract_semantic_info(node: &wdl_grammar::SyntaxNode) -> WdlInfo {
    let mut info = WdlInfo::new();
    collect_semantic_info(node, &mut info);
    info
}

fn collect_semantic_info(node: &wdl_grammar::SyntaxNode, info: &mut WdlInfo) {
    match node.kind() {
        SyntaxKind::VersionStatementNode => {
            for child in node.children_with_tokens() {
                if let Some(token) = child.as_token() {
                    if token.kind() == SyntaxKind::Version {
                        info.version = Some(token.text().to_string());
                        break;
                    }
                }
            }
        }
        SyntaxKind::TaskDefinitionNode => {
            if let Some(task_info) = extract_task_info(&node) {
                info.tasks.push(task_info);
            }
        }
        SyntaxKind::WorkflowDefinitionNode => {
            if let Some(workflow_info) = extract_workflow_info(&node) {
                info.workflows.push(workflow_info);
            }
        }
        SyntaxKind::StructDefinitionNode => {
            if let Some(struct_info) = extract_struct_info(&node) {
                info.structs.push(struct_info);
            }
        }
        SyntaxKind::ImportStatementNode => {
            if let Some(import_info) = extract_import_info(&node) {
                info.imports.push(import_info);
            }
        }
        _ => {}
    }

    // Recursively process child nodes
    for child in node.children() {
        collect_semantic_info(&child, info);
    }
}

fn find_identifier_name(node: &wdl_grammar::SyntaxNode) -> Option<String> {
    for child in node.children_with_tokens() {
        if let Some(token) = child.as_token() {
            if token.kind() == SyntaxKind::Ident {
                return Some(token.text().to_string());
            }
        }
    }
    None
}

fn extract_task_info(node: &wdl_grammar::SyntaxNode) -> Option<TaskInfo> {
    let name = find_identifier_name(&node)?;
    let mut task = TaskInfo {
        name,
        inputs: Vec::new(),
        outputs: Vec::new(),
        command: None,
        runtime: Vec::new(),
        meta: Vec::new(),
        parameter_meta: Vec::new(),
    };

    for child in node.children() {
        match child.kind() {
            SyntaxKind::InputSectionNode => {
                task.inputs.extend(extract_inputs(&child));
            }
            SyntaxKind::OutputSectionNode => {
                task.outputs.extend(extract_outputs(&child));
            }
            SyntaxKind::CommandSectionNode => {
                task.command = Some(extract_command_text(&child));
            }
            SyntaxKind::RuntimeSectionNode => {
                task.runtime.extend(extract_runtime_items(&child));
            }
            SyntaxKind::MetadataSectionNode => {
                task.meta.extend(extract_meta_items(&child));
            }
            SyntaxKind::ParameterMetadataSectionNode => {
                task.parameter_meta.extend(extract_meta_items(&child));
            }
            _ => {}
        }
    }

    Some(task)
}

fn extract_workflow_info(node: &wdl_grammar::SyntaxNode) -> Option<WorkflowInfo> {
    let name = find_identifier_name(&node)?;
    let mut workflow = WorkflowInfo {
        name,
        inputs: Vec::new(),
        outputs: Vec::new(),
        calls: Vec::new(),
        meta: Vec::new(),
        parameter_meta: Vec::new(),
    };

    for child in node.children() {
        match child.kind() {
            SyntaxKind::InputSectionNode => {
                workflow.inputs.extend(extract_inputs(&child));
            }
            SyntaxKind::OutputSectionNode => {
                workflow.outputs.extend(extract_outputs(&child));
            }
            SyntaxKind::CallStatementNode => {
                if let Some(call) = extract_call_info(&child) {
                    workflow.calls.push(call);
                }
            }
            SyntaxKind::MetadataSectionNode => {
                workflow.meta.extend(extract_meta_items(&child));
            }
            SyntaxKind::ParameterMetadataSectionNode => {
                workflow.parameter_meta.extend(extract_meta_items(&child));
            }
            _ => {}
        }
    }

    Some(workflow)
}

fn extract_struct_info(node: &wdl_grammar::SyntaxNode) -> Option<StructInfo> {
    let name = find_identifier_name(&node)?;
    let mut struct_info = StructInfo {
        name,
        fields: Vec::new(),
    };

    for child in node.children() {
        if matches!(
            child.kind(),
            SyntaxKind::UnboundDeclNode | SyntaxKind::BoundDeclNode
        ) {
            if let Some(input) = extract_declaration(&child) {
                struct_info.fields.push(input);
            }
        }
    }

    Some(struct_info)
}

fn extract_import_info(node: &wdl_grammar::SyntaxNode) -> Option<ImportInfo> {
    let mut import = ImportInfo {
        uri: String::new(),
        alias: None,
    };

    for child in node.children() {
        if child.kind() == SyntaxKind::LiteralStringNode {
            for string_child in child.children_with_tokens() {
                if let Some(token) = string_child.as_token() {
                    if token.kind() == SyntaxKind::LiteralStringText {
                        import.uri = token.text().to_string();
                        break;
                    }
                }
            }
        }
    }

    // Look for alias
    let mut found_as = false;
    for child in node.children_with_tokens() {
        if let Some(token) = child.as_token() {
            if token.kind() == SyntaxKind::AsKeyword {
                found_as = true;
            } else if found_as && token.kind() == SyntaxKind::Ident {
                import.alias = Some(token.text().to_string());
                break;
            }
        }
    }

    if import.uri.is_empty() {
        None
    } else {
        Some(import)
    }
}

fn extract_inputs(node: &wdl_grammar::SyntaxNode) -> Vec<InputInfo> {
    let mut inputs = Vec::new();
    for child in node.children() {
        if matches!(
            child.kind(),
            SyntaxKind::UnboundDeclNode | SyntaxKind::BoundDeclNode
        ) {
            if let Some(input) = extract_declaration(&child) {
                inputs.push(input);
            }
        }
    }
    inputs
}

fn extract_outputs(node: &wdl_grammar::SyntaxNode) -> Vec<OutputInfo> {
    let mut outputs = Vec::new();
    for child in node.children() {
        if child.kind() == SyntaxKind::BoundDeclNode {
            if let Some(output) = extract_output_declaration(&child) {
                outputs.push(output);
            }
        }
    }
    outputs
}

fn extract_declaration(node: &wdl_grammar::SyntaxNode) -> Option<InputInfo> {
    let mut input = InputInfo {
        name: String::new(),
        wdl_type: String::new(),
        optional: false,
        default_value: None,
    };

    // Find type and name
    for child in node.children() {
        match child.kind() {
            SyntaxKind::PrimitiveTypeNode
            | SyntaxKind::ArrayTypeNode
            | SyntaxKind::MapTypeNode
            | SyntaxKind::PairTypeNode
            | SyntaxKind::ObjectTypeNode
            | SyntaxKind::TypeRefNode => {
                input.wdl_type = child.text().to_string();
                input.optional = child.text().contains_char('?');
            }
            _ => {}
        }
    }

    // Find name
    if let Some(name) = find_identifier_name(&node) {
        input.name = name;
    }

    // For bound declarations, find default value
    if node.kind() == SyntaxKind::BoundDeclNode {
        // Find the expression after the assignment
        let mut found_assignment = false;
        for child in node.children() {
            if found_assignment {
                input.default_value = Some(child.text().to_string());
                break;
            }
            // Look for assignment token in children_with_tokens
            for token_child in child.children_with_tokens() {
                if let Some(token) = token_child.as_token() {
                    if token.kind() == SyntaxKind::Assignment {
                        found_assignment = true;
                        break;
                    }
                }
            }
        }

        // Alternative approach - look through all tokens
        if input.default_value.is_none() {
            let mut found_assignment = false;
            for child in node.children_with_tokens() {
                if let Some(token) = child.as_token() {
                    if token.kind() == SyntaxKind::Assignment {
                        found_assignment = true;
                    }
                } else if found_assignment {
                    if let Some(child_node) = child.as_node() {
                        input.default_value = Some(child_node.text().to_string());
                        break;
                    }
                }
            }
        }
    }

    if input.name.is_empty() {
        None
    } else {
        Some(input)
    }
}

fn extract_output_declaration(node: &wdl_grammar::SyntaxNode) -> Option<OutputInfo> {
    let mut output = OutputInfo {
        name: String::new(),
        wdl_type: String::new(),
        expression: String::new(),
    };

    // Find type and name
    for child in node.children() {
        match child.kind() {
            SyntaxKind::PrimitiveTypeNode
            | SyntaxKind::ArrayTypeNode
            | SyntaxKind::MapTypeNode
            | SyntaxKind::PairTypeNode
            | SyntaxKind::ObjectTypeNode
            | SyntaxKind::TypeRefNode => {
                output.wdl_type = child.text().to_string();
            }
            _ => {}
        }
    }

    // Find name
    if let Some(name) = find_identifier_name(&node) {
        output.name = name;
    }

    // Find expression after assignment
    let mut found_assignment = false;
    for child in node.children_with_tokens() {
        if let Some(token) = child.as_token() {
            if token.kind() == SyntaxKind::Assignment {
                found_assignment = true;
            }
        } else if found_assignment {
            if let Some(child_node) = child.as_node() {
                output.expression = child_node.text().to_string();
                break;
            }
        }
    }

    if output.name.is_empty() {
        None
    } else {
        Some(output)
    }
}

fn extract_command_text(node: &wdl_grammar::SyntaxNode) -> String {
    let mut command_parts = Vec::new();
    for child in node.children_with_tokens() {
        if let Some(token) = child.as_token() {
            match token.kind() {
                SyntaxKind::LiteralCommandText => {
                    command_parts.push(token.text().to_string());
                }
                _ => {}
            }
        } else if let Some(child_node) = child.as_node() {
            if child_node.kind() == SyntaxKind::PlaceholderNode {
                command_parts.push(format!("~{{{}}}", extract_placeholder_expr(&child_node)));
            }
        }
    }
    command_parts.join("")
}

fn extract_placeholder_expr(node: &wdl_grammar::SyntaxNode) -> String {
    for child in node.children() {
        if matches!(
            child.kind(),
            SyntaxKind::NameRefExprNode | SyntaxKind::AccessExprNode | SyntaxKind::CallExprNode
        ) {
            return child.text().to_string();
        }
    }
    String::new()
}

fn extract_runtime_items(node: &wdl_grammar::SyntaxNode) -> Vec<RuntimeItem> {
    let mut items = Vec::new();
    for child in node.children() {
        if child.kind() == SyntaxKind::RuntimeItemNode {
            if let Some(item) = extract_runtime_item(&child) {
                items.push(item);
            }
        }
    }
    items
}

fn extract_runtime_item(node: &wdl_grammar::SyntaxNode) -> Option<RuntimeItem> {
    let mut key = String::new();
    let mut value = String::new();
    let mut found_colon = false;

    for child in node.children_with_tokens() {
        if let Some(token) = child.as_token() {
            match token.kind() {
                SyntaxKind::Ident if key.is_empty() => {
                    key = token.text().to_string();
                }
                SyntaxKind::Colon => {
                    found_colon = true;
                }
                _ => {}
            }
        } else if found_colon && value.is_empty() {
            if let Some(child_node) = child.as_node() {
                value = child_node.text().to_string();
            }
        }
    }

    if key.is_empty() || value.is_empty() {
        None
    } else {
        Some(RuntimeItem { key, value })
    }
}

fn extract_meta_items(node: &wdl_grammar::SyntaxNode) -> Vec<MetaItem> {
    let mut items = Vec::new();
    for child in node.children() {
        if child.kind() == SyntaxKind::MetadataObjectItemNode {
            if let Some(item) = extract_meta_item(&child) {
                items.push(item);
            }
        }
    }
    items
}

fn extract_meta_item(node: &wdl_grammar::SyntaxNode) -> Option<MetaItem> {
    let mut key = String::new();
    let mut value = String::new();
    let mut found_colon = false;

    for child in node.children_with_tokens() {
        if let Some(token) = child.as_token() {
            match token.kind() {
                SyntaxKind::Ident if key.is_empty() => {
                    key = token.text().to_string();
                }
                SyntaxKind::Colon => {
                    found_colon = true;
                }
                _ => {}
            }
        } else if found_colon && value.is_empty() {
            if let Some(child_node) = child.as_node() {
                value = child_node.text().to_string();
            }
        }
    }

    if key.is_empty() || value.is_empty() {
        None
    } else {
        Some(MetaItem { key, value })
    }
}

fn extract_call_info(node: &wdl_grammar::SyntaxNode) -> Option<CallInfo> {
    let mut call = CallInfo {
        name: String::new(),
        target: String::new(),
        alias: None,
        inputs: Vec::new(),
    };

    for child in node.children() {
        match child.kind() {
            SyntaxKind::CallTargetNode => {
                if let Some(name) = find_identifier_name(&child) {
                    call.target = name.clone();
                    call.name = name;
                }
            }
            SyntaxKind::CallAliasNode => {
                if let Some(alias) = find_identifier_name(&child) {
                    call.alias = Some(alias.clone());
                    call.name = alias;
                }
            }
            SyntaxKind::CallInputItemNode => {
                if let Some(input_item) = extract_call_input_item(&child) {
                    call.inputs.push(input_item);
                }
            }
            _ => {}
        }
    }

    if call.target.is_empty() {
        None
    } else {
        Some(call)
    }
}

fn extract_call_input_item(node: &wdl_grammar::SyntaxNode) -> Option<CallInputItem> {
    let mut name = String::new();
    let mut value = String::new();
    let mut found_assignment = false;

    for child in node.children_with_tokens() {
        if let Some(token) = child.as_token() {
            match token.kind() {
                SyntaxKind::Ident if name.is_empty() => {
                    name = token.text().to_string();
                }
                SyntaxKind::Assignment => {
                    found_assignment = true;
                }
                _ => {}
            }
        } else if found_assignment && value.is_empty() {
            if let Some(child_node) = child.as_node() {
                value = child_node.text().to_string();
            }
        }
    }

    if name.is_empty() || value.is_empty() {
        None
    } else {
        Some(CallInputItem { name, value })
    }
}
