// TODO: fixme

use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

use crate::rule::ProjectRule;
use crate::rust_project::RustProject;

pub struct MustNotHaveCircularDependencies {}

impl Display for MustNotHaveCircularDependencies {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Must not have circular dependencies")
    }
}

impl ProjectRule for MustNotHaveCircularDependencies {
    fn apply(&self, project: &RustProject) -> Result<(), String> {
        if let Some(cycle_path) = find_cycle_in_dependencies(&project.to_dependency_graph()) {
            return Err(format!("Circular dependency cycle detected: {}", cycle_path));
        }

        Ok(())
    }
}

fn find_cycle_in_dependencies(graph: &HashMap<String, Vec<String>>) -> Option<String> {
    let mut visited = HashSet::new();
    let mut current_path = Vec::new();

    for node in graph.keys() {
        if visited.contains(node) {
            continue;
        }

        if let Some(cycle) = dfs_detect_cycle(node, graph, &mut visited, &mut current_path) {
            return Some(cycle);
        }
    }

    None
}

/// DFS ricorsiva per rilevare cicli.
/// - `current_path` tiene traccia del percorso di nodi esplorati nel ramo corrente
/// - Se troviamo un nodo già presente in `current_path`, allora c'è un ciclo
fn dfs_detect_cycle(
    current: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    current_path: &mut Vec<String>,
) -> Option<String> {
    visited.insert(current.to_string());
    current_path.push(current.to_string());

    if let Some(neighbors) = graph.get(current) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                // Continua la DFS
                if let Some(cycle) = dfs_detect_cycle(neighbor, graph, visited, current_path) {
                    return Some(cycle);
                }
            } else if current_path.contains(neighbor) {
                // Trovato ciclo!
                let cycle_start = current_path.iter().position(|x| x == neighbor).unwrap_or(0);
                let cycle_path = current_path[cycle_start..].join(" -> ");
                return Some(format!("{} -> {}", cycle_path, neighbor));
            }
        }
    }

    // Uscita dalla ricorsione
    current_path.pop();
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_dfs_detect_cycle_no_cycle() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("main".to_string(), vec!["lib".to_string()]);
        graph.insert("lib".to_string(), vec!["utils".to_string()]);
        graph.insert("utils".to_string(), vec![]);

        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        let result = dfs_detect_cycle("main", &graph, &mut visited, &mut current_path);

        // Non ci deve essere nessun ciclo
        assert!(result.is_none(), "Expected no cycle, but found one.");
    }

    #[test]
    fn test_dfs_detect_cycle_with_cycle() {
        // (main -> lib -> utils -> main)
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("main".to_string(), vec!["lib".to_string()]);
        graph.insert("lib".to_string(), vec!["utils".to_string()]);
        graph.insert("utils".to_string(), vec!["main".to_string()]);

        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        let result = dfs_detect_cycle("main", &graph, &mut visited, &mut current_path);

        assert!(result.is_some(), "Expected a cycle, but none was found.");
        assert_eq!(
            result.unwrap(),
            "main -> lib -> utils -> main",
            "The detected cycle does not match the expected one."
        );
    }

    #[test]
    fn test_dfs_detect_cycle_complex_graph_with_cycle() {
        // (a -> b -> c -> d -> b)
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("a".to_string(), vec!["b".to_string(), "e".to_string()]);
        graph.insert("b".to_string(), vec!["c".to_string()]);
        graph.insert("c".to_string(), vec!["d".to_string()]);
        graph.insert("d".to_string(), vec!["b".to_string()]);
        graph.insert("e".to_string(), vec!["f".to_string()]);
        graph.insert("f".to_string(), vec![]);

        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        let result = dfs_detect_cycle("a", &graph, &mut visited, &mut current_path);

        assert!(result.is_some(), "Expected a cycle, but none was found.");
        assert_eq!(
            result.unwrap(),
            "b -> c -> d -> b",
            "The detected cycle does not match the expected one."
        );
    }

    #[test]
    fn test_dfs_detect_cycle_complex_graph_no_cycle() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("a".to_string(), vec!["b".to_string(), "e".to_string()]);
        graph.insert("b".to_string(), vec!["c".to_string()]);
        graph.insert("c".to_string(), vec!["d".to_string()]);
        graph.insert("d".to_string(), vec![]);
        graph.insert("e".to_string(), vec!["f".to_string()]);
        graph.insert("f".to_string(), vec![]);

        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        let result = dfs_detect_cycle("a", &graph, &mut visited, &mut current_path);

        assert!(result.is_none(), "Expected no cycle, but found one.");
    }
}
