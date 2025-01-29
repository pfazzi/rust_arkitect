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
        let graph = project.to_dependency_graph();

        if let Some(cycle_path) = find_cycle_in_dependencies(&graph) {
            return Err(format!(
                "Circular dependency cycle detected: {}",
                cycle_path
            ));
        }
        Ok(())
    }
}

/// Cerca eventuali cicli nel grafo *dopo* aver unificato submoduli:
pub fn find_cycle_in_dependencies(graph: &HashMap<String, Vec<String>>) -> Option<String> {
    // 1) Unifichiamo i nodi se necessario
    let unified_graph = unify_submodules_in_graph(graph);

    // 2) Creiamo i nodi locali
    let mut nodes: Vec<&str> = unified_graph.keys().map(|k| k.as_str()).collect();
    nodes.sort();
    let node_index: HashMap<&str, usize> = nodes.iter().enumerate().map(|(i, &n)| (n, i)).collect();

    // 3) Build adjacency
    let adjacency_list = build_adjacency_list(&unified_graph, &node_index);

    // 4) Tarjan
    let sccs = tarjan_scc(&adjacency_list);

    // 5) Cerca la prima SCC di dim > 1
    for scc in sccs {
        if scc.len() > 1 {
            // Ricostruisci un ciclo
            let cycle_path = reconstruct_cycle_path(&scc, &nodes, &adjacency_list);
            return Some(cycle_path);
        }
    }
    None
}

/// Costruisce la lista di adiacenza numerica per Tarjan
fn build_adjacency_list(
    graph: &HashMap<String, Vec<String>>,
    node_index: &HashMap<&str, usize>,
) -> Vec<Vec<usize>> {
    let mut adjacency_list = vec![vec![]; node_index.len()];

    for (node, deps) in graph {
        let i = node_index[node.as_str()];
        for d in deps {
            if let Some(&j) = node_index.get(d.as_str()) {
                adjacency_list[i].push(j);
            }
        }
    }

    adjacency_list
}

/// Esegue l'algoritmo di Tarjan per le Strongly Connected Components.
/// Restituisce un vettore di SCC, ognuna delle quali è un vettore di indici di nodi.
fn tarjan_scc(adjacency_list: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = adjacency_list.len();
    let mut index = 0;
    let mut stack = Vec::new();
    let mut in_stack = vec![false; n];
    let mut indices = vec![-1; n];
    let mut lowlink = vec![-1; n];
    let mut sccs = Vec::new();

    fn strongconnect(
        v: usize,
        index: &mut i32,
        stack: &mut Vec<usize>,
        in_stack: &mut Vec<bool>,
        indices: &mut Vec<i32>,
        lowlink: &mut Vec<i32>,
        adjacency_list: &[Vec<usize>],
        sccs: &mut Vec<Vec<usize>>,
    ) {
        *index += 1;
        indices[v] = *index;
        lowlink[v] = *index;
        stack.push(v);
        in_stack[v] = true;

        for &w in &adjacency_list[v] {
            if indices[w] == -1 {
                strongconnect(
                    w,
                    index,
                    stack,
                    in_stack,
                    indices,
                    lowlink,
                    adjacency_list,
                    sccs,
                );
                lowlink[v] = lowlink[v].min(lowlink[w]);
            } else if in_stack[w] {
                lowlink[v] = lowlink[v].min(indices[w]);
            }
        }

        if lowlink[v] == indices[v] {
            let mut scc = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                in_stack[w] = false;
                scc.push(w);
                if w == v {
                    break;
                }
            }
            sccs.push(scc);
        }
    }

    for v in 0..n {
        if indices[v] == -1 {
            strongconnect(
                v,
                &mut index,
                &mut stack,
                &mut in_stack,
                &mut indices,
                &mut lowlink,
                adjacency_list,
                &mut sccs,
            );
        }
    }

    sccs
}

/// Ricostruisce un percorso ciclico da una SCC di dimensione > 1.
fn reconstruct_cycle_path(scc: &[usize], nodes: &[&str], adjacency_list: &[Vec<usize>]) -> String {
    let scc_set: HashSet<usize> = scc.iter().cloned().collect();
    let start = scc[0];

    let mut path = Vec::new();
    let mut visited = HashSet::new();
    if let Some(cycle) = dfs_cycle_reconstruct(
        start,
        start,
        adjacency_list,
        &scc_set,
        &mut path,
        &mut visited,
        nodes,
    ) {
        return cycle;
    }
    "Unable to reconstruct cycle".to_string()
}

/// DFS per ricostruire il ciclo effettivo come sequenza di nomi
fn dfs_cycle_reconstruct(
    current: usize,
    start: usize,
    adjacency_list: &[Vec<usize>],
    scc_set: &HashSet<usize>,
    path: &mut Vec<usize>,
    visited: &mut HashSet<usize>,
    nodes: &[&str],
) -> Option<String> {
    visited.insert(current);
    path.push(current);

    for &next in &adjacency_list[current] {
        if !scc_set.contains(&next) {
            continue;
        }
        if next == start && path.len() > 1 {
            return Some(translate_cycle_to_names(path, nodes));
        }
        if !visited.contains(&next) {
            if let Some(cycle) =
                dfs_cycle_reconstruct(next, start, adjacency_list, scc_set, path, visited, nodes)
            {
                return Some(cycle);
            }
        }
    }

    path.pop();
    None
}

/// Converte gli indici in nomi
fn translate_cycle_to_names(path: &[usize], nodes: &[&str]) -> String {
    if path.is_empty() {
        return "".to_string();
    }
    let mut names: Vec<&str> = path.iter().map(|&ix| nodes[ix]).collect();
    names.push(nodes[path[0]]);
    names.join(" -> ")
}

/// Unifica i submoduli in un grafo basato su un criterio (ad esempio, tutto ciò che inizia con un prefisso comune).
pub fn unify_submodules_in_graph(
    original_graph: &HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    let mut new_graph = HashMap::new();

    for (node, deps) in original_graph {
        let unified_node = unify_container_submodules(node);

        let unified_deps: Vec<String> =
            deps.iter().map(|d| unify_container_submodules(d)).collect();

        new_graph
            .entry(unified_node)
            .or_insert_with(Vec::new)
            .extend(unified_deps);
    }

    for deps in new_graph.values_mut() {
        deps.sort();
        deps.dedup();
    }

    new_graph
}

/// Unifica un singolo nodo, trattando i sotto-moduli come un nodo principale.
fn unify_container_submodules(node: &str) -> String {
    let prefix = "crate::application::container::";
    if node.starts_with(prefix) {
        "crate::application::container".to_string()
    } else {
        node.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_no_cycle_in_hexagonal_architecture() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        graph.insert("domain::policy".to_string(), vec![]);
        graph.insert("domain::quote".to_string(), vec![]);
        graph.insert(
            "application::policy_service".to_string(),
            vec!["domain::policy".to_string()],
        );
        graph.insert(
            "application::quote_service".to_string(),
            vec!["domain::quote".to_string()],
        );
        graph.insert(
            "infrastructure::database::policy_repository".to_string(),
            vec!["domain::policy".to_string()],
        );
        graph.insert(
            "infrastructure::external::quote_api".to_string(),
            vec!["application::quote_service".to_string()],
        );

        let result = find_cycle_in_dependencies(&graph);
        assert!(
            result.is_none(),
            "Expected no cycles, but found one: {:?}",
            result
        );
    }

    #[test]
    fn test_direct_cycle_between_two_modules() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("module_a".to_string(), vec!["module_b".to_string()]);
        graph.insert("module_b".to_string(), vec!["module_a".to_string()]);

        let result = find_cycle_in_dependencies(&graph);
        assert!(result.is_some(), "Expected a cycle, but none was found.");
    }

    #[test]
    fn test_indirect_cycle_across_three_modules() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("module_a".to_string(), vec!["module_b".to_string()]);
        graph.insert("module_b".to_string(), vec!["module_c".to_string()]);
        graph.insert("module_c".to_string(), vec!["module_a".to_string()]);

        let result = find_cycle_in_dependencies(&graph);
        assert!(
            result.is_some(),
            "Expected an indirect cycle, but none was found."
        );
    }

    #[test]
    fn test_no_cycles_with_multiple_independent_paths() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert(
            "module_a".to_string(),
            vec!["module_b".to_string(), "module_c".to_string()],
        );
        graph.insert("module_b".to_string(), vec!["module_d".to_string()]);
        graph.insert("module_c".to_string(), vec!["module_d".to_string()]);
        graph.insert("module_d".to_string(), vec![]);

        let result = find_cycle_in_dependencies(&graph);
        assert!(
            result.is_none(),
            "Expected no cycles, but found one: {:?}",
            result
        );
    }

    #[test]
    fn test_complex_cycle_with_multiple_entries() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert("module_a".to_string(), vec!["module_b".to_string()]);
        graph.insert("module_b".to_string(), vec!["module_c".to_string()]);
        graph.insert("module_c".to_string(), vec!["module_d".to_string()]);
        graph.insert("module_d".to_string(), vec!["module_a".to_string()]);
        graph.insert("module_x".to_string(), vec!["module_y".to_string()]);
        graph.insert("module_y".to_string(), vec![]);

        let result = find_cycle_in_dependencies(&graph);
        assert!(result.is_some(), "Expected a cycle, but none was found.");
    }

    #[test]
    fn test_no_dependency_from_domain_to_other_modules() {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        graph.insert(
            "domain::policy".to_string(),
            vec!["application::policy_service".to_string()],
        );
        graph.insert(
            "application::policy_service".to_string(),
            vec!["domain::policy".to_string()],
        );
        graph.insert(
            "infrastructure::database::policy_repository".to_string(),
            vec!["domain::policy".to_string()],
        );

        let result = find_cycle_in_dependencies(&graph);
        assert!(
            result.is_some(),
            "Expected a dependency from domain to application, but none was found."
        );
    }
}
