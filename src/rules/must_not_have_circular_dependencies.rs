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
    /// Applica la regola al progetto, verificando che il grafo di dipendenze
    /// non contenga alcun ciclo.
    fn apply(&self, project: &RustProject) -> Result<(), String> {
        if let Some(cycle_path) = find_cycle_in_dependencies(&project.to_dependency_graph()) {
            return Err(format!("Rilevata dipendenza circolare: {}", cycle_path));
        }
        Ok(())
    }
}

/// Cerca cicli nel grafo di dipendenze.
///
/// Se ne trova uno, ritorna `Some(path)`, dove `path` descrive la catena del ciclo.
/// Altrimenti, ritorna `None`.
fn find_cycle_in_dependencies(graph: &HashMap<String, Vec<String>>) -> Option<String> {
    let mut visited = HashSet::new();
    let mut current_path = Vec::new();

    // Proviamo a partire da ogni nodo
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
