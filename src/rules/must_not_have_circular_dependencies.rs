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
            return Err(format!(
                "Circular dependency cycle detected: {}",
                cycle_path
            ));
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
    fn test_realistic_no_cycle() {
        // Un grafo realistico senza cicli
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert(
            "crate::application::container".to_string(),
            vec![
                "crate::application::geographic_info".to_string(),
                "crate::domain::aggregate::quote".to_string(),
            ],
        );
        graph.insert(
            "crate::application::geographic_info".to_string(),
            vec![
                "crate::infrastructure::bridge::antifraud".to_string(),
                "crate::infrastructure::bridge::payment".to_string(),
            ],
        );
        graph.insert("crate::domain::aggregate::quote".to_string(), vec![]);
        graph.insert(
            "crate::infrastructure::bridge::antifraud".to_string(),
            vec![],
        );
        graph.insert("crate::infrastructure::bridge::payment".to_string(), vec![]);

        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        let result = dfs_detect_cycle(
            "crate::application::container",
            &graph,
            &mut visited,
            &mut current_path,
        );

        // Non ci deve essere alcun ciclo
        assert!(result.is_none(), "Expected no cycle, but one was found.");
    }

    #[test]
    fn test_realistic_with_cycle() {
        // Un grafo realistico con un ciclo tra i moduli
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert(
            "crate::application::container".to_string(),
            vec![
                "crate::application::geographic_info".to_string(),
                "crate::domain::aggregate::quote".to_string(),
            ],
        );
        graph.insert(
            "crate::application::geographic_info".to_string(),
            vec!["crate::infrastructure::bridge::payment".to_string()],
        );
        graph.insert(
            "crate::infrastructure::bridge::payment".to_string(),
            vec!["crate::application::container".to_string()], // Crea il ciclo
        );
        graph.insert("crate::domain::aggregate::quote".to_string(), vec![]);

        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        let result = dfs_detect_cycle(
            "crate::application::container",
            &graph,
            &mut visited,
            &mut current_path,
        );

        // Deve trovare un ciclo
        assert!(result.is_some(), "Expected a cycle, but none was found.");
        assert_eq!(
            result.unwrap(),
            "crate::application::container -> crate::application::geographic_info -> crate::infrastructure::bridge::payment -> crate::application::container",
            "The detected cycle does not match the expected one."
        );
    }

    #[test]
    fn test_realistic_large_graph_no_cycle() {
        // Un grafo grande senza cicli
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert(
            "crate::application::container".to_string(),
            vec![
                "crate::application::geographic_info".to_string(),
                "crate::domain::aggregate::quote".to_string(),
                "crate::domain::price".to_string(),
            ],
        );
        graph.insert(
            "crate::application::geographic_info".to_string(),
            vec![
                "crate::infrastructure::bridge::payment".to_string(),
                "crate::infrastructure::bridge::s3_service".to_string(),
            ],
        );
        graph.insert(
            "crate::domain::price".to_string(),
            vec!["crate::domain::aggregate::quote".to_string()],
        );
        graph.insert("crate::domain::aggregate::quote".to_string(), vec![]);
        graph.insert("crate::infrastructure::bridge::payment".to_string(), vec![]);
        graph.insert(
            "crate::infrastructure::bridge::s3_service".to_string(),
            vec![],
        );

        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        let result = dfs_detect_cycle(
            "crate::application::container",
            &graph,
            &mut visited,
            &mut current_path,
        );

        // Non ci deve essere alcun ciclo
        assert!(result.is_none(), "Expected no cycle, but one was found.");
    }

    #[test]
    fn test_realistic_large_graph_with_complex_cycle() {
        // Un grafo grande con un ciclo complesso
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert(
            "crate::application::container".to_string(),
            vec![
                "crate::application::geographic_info".to_string(),
                "crate::domain::aggregate::quote".to_string(),
            ],
        );
        graph.insert(
            "crate::application::geographic_info".to_string(),
            vec!["crate::domain::price".to_string()],
        );
        graph.insert(
            "crate::domain::price".to_string(),
            vec!["crate::domain::aggregate::quote".to_string()],
        );
        graph.insert(
            "crate::domain::aggregate::quote".to_string(),
            vec!["crate::application::container".to_string()], // Crea il ciclo
        );

        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        let result = dfs_detect_cycle(
            "crate::application::container",
            &graph,
            &mut visited,
            &mut current_path,
        );

        // Deve trovare un ciclo
        assert!(result.is_some(), "Expected a cycle, but none was found.");
        assert_eq!(
            result.unwrap(),
            "crate::application::container -> crate::application::geographic_info -> crate::domain::price -> crate::domain::aggregate::quote -> crate::application::container",
            "The detected cycle does not match the expected one."
        );
    }
}
