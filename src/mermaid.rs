use std::collections::{HashMap, HashSet};
use wdl_grammar::{SyntaxKind, SyntaxNode, SyntaxTree};

pub struct WorkflowGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    node_ids: HashSet<String>,
}

pub struct Node {
    id: String,
    label: String,
    node_type: NodeType,
}

pub struct Edge {
    from: String,
    to: String,
    label: Option<String>,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Task,
    Input,
    Output,
    Conditional,
    Scatter,
    Workflow,
    Call,
}

impl WorkflowGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_ids: HashSet::new(),
        }
    }

    pub fn add_node(&mut self, id: String, label: String, node_type: NodeType) {
        if !self.node_ids.contains(&id) {
            self.node_ids.insert(id.clone());
            self.nodes.push(Node {
                id,
                label,
                node_type,
            });
        }
    }

    pub fn add_edge(&mut self, from: String, to: String, label: Option<String>) {
        self.edges.push(Edge { from, to, label });
    }
}

// Parse WDL and extract workflow structure
pub fn extract_workflow_graph(content: &str) -> Result<WorkflowGraph, String> {
    let (tree, _diagnostics) = SyntaxTree::parse(content);
    let root = tree.root();

    let mut graph = WorkflowGraph::new();
    let mut call_dependencies = HashMap::<String, Vec<String>>::new();

    // Walk the AST and extract workflow information
    walk_node(&root, &mut graph, &mut call_dependencies);

    // Add dependency edges
    add_dependency_edges(&mut graph, &call_dependencies);

    Ok(graph)
}

fn walk_node(
    node: &SyntaxNode,
    graph: &mut WorkflowGraph,
    dependencies: &mut HashMap<String, Vec<String>>,
) {
    walk_node_with_context(node, graph, dependencies, None);
}

fn walk_node_with_context(
    node: &SyntaxNode,
    graph: &mut WorkflowGraph,
    dependencies: &mut HashMap<String, Vec<String>>,
    current_workflow: Option<String>,
) {
    match node.kind() {
        SyntaxKind::WorkflowDefinitionNode => {
            if let Some(name) = find_workflow_name(node) {
                let workflow_id = format!("workflow_{}", name);
                graph.add_node(workflow_id.clone(), name.clone(), NodeType::Workflow);

                // Process workflow inputs
                if let Some(input_section) = find_child_by_kind(node, SyntaxKind::InputSectionNode)
                {
                    process_input_section(&input_section, graph, &workflow_id);
                }

                // Process workflow outputs
                if let Some(output_section) =
                    find_child_by_kind(node, SyntaxKind::OutputSectionNode)
                {
                    process_output_section(&output_section, graph, &workflow_id);
                }

                // Recursively process children with workflow context
                for child in node.children() {
                    walk_node_with_context(&child, graph, dependencies, Some(workflow_id.clone()));
                }
                return; // Don't process children again below
            }
        }
        SyntaxKind::TaskDefinitionNode => {
            if let Some(name) = find_task_name(node) {
                graph.add_node(format!("task_{}", name), name, NodeType::Task);
            }
        }
        SyntaxKind::CallStatementNode => {
            process_call_statement(node, graph, dependencies, current_workflow.as_ref());
        }
        SyntaxKind::ConditionalStatementNode => {
            process_conditional_statement(node, graph, dependencies, current_workflow.as_ref());
        }
        SyntaxKind::ScatterStatementNode => {
            process_scatter_statement(node, graph, dependencies, current_workflow.as_ref());
        }
        _ => {}
    }

    // Recursively process children (if not already processed above)
    if !matches!(node.kind(), SyntaxKind::WorkflowDefinitionNode) {
        for child in node.children() {
            walk_node_with_context(&child, graph, dependencies, current_workflow.clone());
        }
    }
}

fn find_workflow_name(node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens().find_map(|child| {
        if let Some(token) = child.as_token() {
            if token.kind() == SyntaxKind::Ident {
                Some(token.text().to_string())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn find_task_name(node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens().find_map(|child| {
        if let Some(token) = child.as_token() {
            if token.kind() == SyntaxKind::Ident {
                Some(token.text().to_string())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn find_child_by_kind(node: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxNode> {
    node.children().find(|child| child.kind() == kind)
}

fn process_input_section(input_section: &SyntaxNode, graph: &mut WorkflowGraph, parent_id: &str) {
    let mut input_count = 0;
    for child in input_section.descendants() {
        if child.kind() == SyntaxKind::BoundDeclNode || child.kind() == SyntaxKind::UnboundDeclNode
        {
            if let Some(var_name) = extract_declaration_name(&child) {
                input_count += 1;
                let input_id = format!("{}_input_{}", parent_id, input_count);
                graph.add_node(
                    input_id.clone(),
                    format!("Input: {}", var_name),
                    NodeType::Input,
                );
                graph.add_edge(input_id, parent_id.to_string(), None);
            }
        }
    }
}

fn process_output_section(output_section: &SyntaxNode, graph: &mut WorkflowGraph, parent_id: &str) {
    let mut output_count = 0;
    for child in output_section.descendants() {
        if child.kind() == SyntaxKind::BoundDeclNode {
            if let Some(var_name) = extract_declaration_name(&child) {
                output_count += 1;
                let output_id = format!("{}_output_{}", parent_id, output_count);
                graph.add_node(
                    output_id.clone(),
                    format!("Output: {}", var_name),
                    NodeType::Output,
                );
                graph.add_edge(parent_id.to_string(), output_id, None);
            }
        }
    }
}

fn extract_declaration_name(decl_node: &SyntaxNode) -> Option<String> {
    decl_node.children_with_tokens().find_map(|child| {
        if let Some(token) = child.as_token() {
            if token.kind() == SyntaxKind::Ident {
                Some(token.text().to_string())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn process_call_statement(
    node: &SyntaxNode,
    graph: &mut WorkflowGraph,
    dependencies: &mut HashMap<String, Vec<String>>,
    parent_workflow: Option<&String>,
) {
    if let Some(call_name) = find_call_name(node) {
        let call_id = format!("call_{}", call_name);
        graph.add_node(
            call_id.clone(),
            format!("call {}", call_name),
            NodeType::Call,
        );

        // Connect call to parent workflow
        if let Some(workflow_id) = parent_workflow {
            graph.add_edge(workflow_id.clone(), call_id.clone(), None);
        }

        // Extract dependencies from call inputs
        let deps = extract_call_dependencies(node);
        if !deps.is_empty() {
            dependencies.insert(call_id, deps);
        }
    }
}

fn find_call_name(node: &SyntaxNode) -> Option<String> {
    // Look for CallTargetNode which contains the identifier
    for child in node.children() {
        if child.kind() == SyntaxKind::CallTargetNode {
            for target_child in child.children_with_tokens() {
                if let Some(token) = target_child.as_token() {
                    if token.kind() == SyntaxKind::Ident {
                        return Some(token.text().to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_call_dependencies(node: &SyntaxNode) -> Vec<String> {
    let mut deps = Vec::new();

    // Look for input assignments that reference other calls
    for child in node.descendants() {
        if child.kind() == SyntaxKind::CallInputItemNode {
            // Look for expressions in the call input
            for expr_child in child.descendants() {
                extract_dependencies_from_expression(&expr_child, &mut deps);
            }
        }
    }

    deps
}

fn extract_dependencies_from_expression(expr: &SyntaxNode, deps: &mut Vec<String>) {
    for child in expr.descendants() {
        // Look for member access patterns like "task_name.output"
        if child.kind() == SyntaxKind::AccessExprNode {
            for access_child in child.children_with_tokens() {
                if let Some(token) = access_child.as_token() {
                    if token.kind() == SyntaxKind::Ident {
                        let name = token.text().to_string();
                        if !deps.contains(&format!("call_{}", name)) {
                            deps.push(format!("call_{}", name));
                        }
                        break; // Only take the first identifier (the task name)
                    }
                }
            }
        }
    }
}

fn process_conditional_statement(
    node: &SyntaxNode,
    graph: &mut WorkflowGraph,
    dependencies: &mut HashMap<String, Vec<String>>,
    parent_workflow: Option<&String>,
) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static CONDITIONAL_COUNTER: AtomicUsize = AtomicUsize::new(0);
    let count = CONDITIONAL_COUNTER.fetch_add(1, Ordering::SeqCst);
    let cond_id = format!("conditional_{}", count + 1);

    graph.add_node(
        cond_id.clone(),
        "if condition".to_string(),
        NodeType::Conditional,
    );

    // Connect conditional to parent workflow
    if let Some(workflow_id) = parent_workflow {
        graph.add_edge(workflow_id.clone(), cond_id.clone(), None);
    }

    // Process statements inside conditional
    for child in node.children() {
        if child.kind() == SyntaxKind::CallStatementNode {
            process_call_statement(&child, graph, dependencies, parent_workflow);
            if let Some(call_name) = find_call_name(&child) {
                graph.add_edge(cond_id.clone(), format!("call_{}", call_name), None);
            }
        }
    }
}

fn process_scatter_statement(
    node: &SyntaxNode,
    graph: &mut WorkflowGraph,
    dependencies: &mut HashMap<String, Vec<String>>,
    parent_workflow: Option<&String>,
) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SCATTER_COUNTER: AtomicUsize = AtomicUsize::new(0);
    let count = SCATTER_COUNTER.fetch_add(1, Ordering::SeqCst);
    let scatter_id = format!("scatter_{}", count + 1);

    // Extract scatter variable
    let scatter_var = node
        .children_with_tokens()
        .find_map(|child| {
            if let Some(token) = child.as_token() {
                if token.kind() == SyntaxKind::Ident {
                    Some(token.text().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| "item".to_string());

    graph.add_node(
        scatter_id.clone(),
        format!("scatter {}", scatter_var),
        NodeType::Scatter,
    );

    // Connect scatter to parent workflow
    if let Some(workflow_id) = parent_workflow {
        graph.add_edge(workflow_id.clone(), scatter_id.clone(), None);
    }

    // Process statements inside scatter
    for child in node.children() {
        if child.kind() == SyntaxKind::CallStatementNode {
            process_call_statement(&child, graph, dependencies, parent_workflow);
            if let Some(call_name) = find_call_name(&child) {
                graph.add_edge(scatter_id.clone(), format!("call_{}", call_name), None);
            }
        }
    }
}

fn add_dependency_edges(graph: &mut WorkflowGraph, dependencies: &HashMap<String, Vec<String>>) {
    for (call_id, deps) in dependencies {
        for dep in deps {
            graph.add_edge(dep.clone(), call_id.clone(), Some("depends on".to_string()));
        }
    }
}

// Convert to Mermaid format
pub fn generate_mermaid(graph: &WorkflowGraph) -> String {
    let mut mermaid = String::from("flowchart TD\n");

    // Add nodes with styling based on type
    for node in &graph.nodes {
        let shape_and_style = match node.node_type {
            NodeType::Task => (format!("{}[{}]", node.id, node.label), ":::taskStyle"),
            NodeType::Call => (format!("{}[{}]", node.id, node.label), ":::callStyle"),
            NodeType::Input => (format!("{}(({}))", node.id, node.label), ":::inputStyle"),
            NodeType::Output => (format!("{}(({}))", node.id, node.label), ":::outputStyle"),
            NodeType::Conditional => (
                format!("{}{{/{}/}}", node.id, node.label),
                ":::conditionalStyle",
            ),
            NodeType::Scatter => (format!("{}[/{}\\]", node.id, node.label), ":::scatterStyle"),
            NodeType::Workflow => (format!("{}([{}])", node.id, node.label), ":::workflowStyle"),
        };

        mermaid.push_str(&format!("    {}{}\n", shape_and_style.0, shape_and_style.1));
    }

    // Add edges
    for edge in &graph.edges {
        match &edge.label {
            Some(label) => {
                mermaid.push_str(&format!("    {} ---|{}| {}\n", edge.from, label, edge.to))
            }
            None => mermaid.push_str(&format!("    {} --> {}\n", edge.from, edge.to)),
        }
    }

    // Add styling
    mermaid.push_str("\n");
    mermaid.push_str("    classDef taskStyle fill:#e1f5fe,stroke:#01579b,stroke-width:2px\n");
    mermaid.push_str("    classDef callStyle fill:#f3e5f5,stroke:#4a148c,stroke-width:2px\n");
    mermaid.push_str("    classDef inputStyle fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px\n");
    mermaid.push_str("    classDef outputStyle fill:#fff3e0,stroke:#ef6c00,stroke-width:2px\n");
    mermaid
        .push_str("    classDef conditionalStyle fill:#fff8e1,stroke:#f57f17,stroke-width:2px\n");
    mermaid.push_str("    classDef scatterStyle fill:#fce4ec,stroke:#c2185b,stroke-width:2px\n");
    mermaid.push_str("    classDef workflowStyle fill:#f1f8e9,stroke:#33691e,stroke-width:3px\n");

    mermaid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_workflow() {
        let wdl_content = r#"
version 1.1

task say_hello {
    input {
        String name
    }
    command {
        echo "Hello ${name}"
    }
    output {
        String greeting = stdout()
    }
}

workflow hello_world {
    input {
        String input_name
    }

    call say_hello {
        input: name = input_name
    }

    output {
        String result = say_hello.greeting
    }
}
"#;

        let graph = extract_workflow_graph(wdl_content).expect("Failed to parse WDL");
        let mermaid = generate_mermaid(&graph);

        assert!(mermaid.contains("flowchart TD"));
        assert!(mermaid.contains("workflow_hello_world"));
        assert!(mermaid.contains("call_say_hello"));
    }

    #[test]
    fn test_mermaid_generation() {
        let mut graph = WorkflowGraph::new();
        graph.add_node("task1".to_string(), "Task 1".to_string(), NodeType::Task);
        graph.add_node("task2".to_string(), "Task 2".to_string(), NodeType::Task);
        graph.add_edge("task1".to_string(), "task2".to_string(), None);

        let mermaid = generate_mermaid(&graph);
        assert!(mermaid.contains("flowchart TD"));
        assert!(mermaid.contains("task1[Task 1]"));
        assert!(mermaid.contains("task1 --> task2"));
    }
}
