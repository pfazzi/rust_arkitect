use crate::rule::ProjectRule;
use crate::rust_project::RustProject;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

pub struct MustNotHaveCircularDependencies {
    pub max_depth: usize,
}

impl Display for MustNotHaveCircularDependencies {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Must not have circular dependencies (max_depth = {})",
            self.max_depth
        )
    }
}

impl ProjectRule for MustNotHaveCircularDependencies {
    fn apply(&self, project: &RustProject) -> Result<(), String> {
        let graph = project.to_dependency_graph();

        let cycles = find_all_cycles_in_dependencies(&graph, self.max_depth);

        if !cycles.is_empty() {
            return Err(format!(
                "Circular dependencies detected:\n{}",
                cycles.join("\n")
            ));
        }

        Ok(())
    }
}

pub fn find_all_cycles_in_dependencies(
    graph: &HashMap<String, Vec<String>>,
    max_depth: usize,
) -> Vec<String> {
    let unified_graph = unify_submodules_in_graph(graph, max_depth);

    let mut nodes: Vec<&str> = unified_graph.keys().map(|k| k.as_str()).collect();
    nodes.sort();
    let node_index: HashMap<&str, usize> = nodes.iter().enumerate().map(|(i, &n)| (n, i)).collect();

    // Build adjacency
    let adjacency_list = build_adjacency_list(&unified_graph, &node_index);

    // Tarjan
    let sccs = tarjan_scc(&adjacency_list);

    let mut cycles = Vec::new();

    // Process each SCC to extract cycles
    for scc in sccs {
        if scc.len() > 1 {
            let cycle_path = reconstruct_cycle_path(&scc, &nodes, &adjacency_list);
            cycles.push(cycle_path);
        } else if scc.len() == 1 {
            let only_node = scc[0];
            if adjacency_list[only_node].contains(&only_node) {
                // Check if it's an actual circular dependency
                if adjacency_list[only_node].len() > 1 {
                    // If it's just a self-reference, ignore it
                    cycles.push(format!("{} -> {}", nodes[only_node], nodes[only_node]));
                }
            }
        }
    }

    cycles
}

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

fn reconstruct_cycle_path(scc: &[usize], nodes: &[&str], adjacency_list: &[Vec<usize>]) -> String {
    if scc.len() == 1 {
        let v = scc[0];
        if adjacency_list[v].contains(&v) {
            if adjacency_list[v].len() == 1 {
                // Ignore pure self-references
                println!("Ignoring reconstruction of self-cycle: {}", nodes[v]);
                return "".to_string();
            }
            return format!("{} -> {}", nodes[v], nodes[v]);
        }
    }

    // Proceed with normal cycle reconstruction...
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

    "".to_string()
}

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

fn translate_cycle_to_names(path: &[usize], nodes: &[&str]) -> String {
    if path.is_empty() {
        return "".to_string();
    }
    let mut names: Vec<&str> = path.iter().map(|&ix| nodes[ix]).collect();
    names.push(nodes[path[0]]);
    names.join(" -> ")
}

pub fn unify_submodules_in_graph(
    original_graph: &HashMap<String, Vec<String>>,
    max_depth: usize,
) -> HashMap<String, Vec<String>> {
    let mut new_graph = HashMap::new();

    for (node, deps) in original_graph {
        let unified_node = unify_submodules(node, max_depth);
        let unified_deps: Vec<String> = deps
            .iter()
            .map(|d| unify_submodules(d, max_depth))
            .collect();

        new_graph
            .entry(unified_node)
            .or_insert_with(Vec::new)
            .extend(unified_deps);
    }

    let all_deps: Vec<String> = new_graph.values().flatten().cloned().collect();
    for dep_node in all_deps {
        new_graph.entry(dep_node).or_insert_with(Vec::new);
    }

    for deps in new_graph.values_mut() {
        deps.sort();
        deps.dedup();
    }

    new_graph
}

fn unify_submodules(node: &str, max_depth: usize) -> String {
    let parts: Vec<&str> = node.split("::").collect();
    if parts.len() <= max_depth {
        node.to_string()
    } else {
        parts[..max_depth].join("::")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    //
    // ----------------------
    // TESTS FOR `unify_submodules`
    // ----------------------
    //

    #[test]
    fn test_unify_submodules_no_truncation() {
        let node = "crate::mod1::mod2";
        let result = unify_submodules(node, 5);
        assert_eq!(result, "crate::mod1::mod2");
    }

    #[test]
    fn test_unify_submodules_exact_match() {
        let node = "crate::mod1::mod2";
        let result = unify_submodules(node, 3);
        assert_eq!(result, "crate::mod1::mod2");
    }

    #[test]
    fn test_unify_submodules_truncate() {
        let node = "crate::application::container::submod";
        let result = unify_submodules(node, 2);
        assert_eq!(result, "crate::application");
    }

    #[test]
    fn test_unify_submodules_zero_depth() {
        let node = "crate::application::submod";
        let result = unify_submodules(node, 0);
        assert_eq!(result, "", "With max_depth=0, we expect an empty string");
    }

    //
    // ----------------------
    // TESTS FOR `unify_submodules_in_graph`
    // ----------------------
    //

    #[test]
    fn test_unify_submodules_in_graph_zero_depth() {
        let mut graph = HashMap::new();
        graph.insert(
            "crate::mod1::sub1".to_string(),
            vec!["crate::mod2::sub2".to_string()],
        );
        let unified = unify_submodules_in_graph(&graph, 0);

        assert_eq!(unified.len(), 1);
        assert!(
            unified.contains_key(""),
            "Empty key created after unification"
        );
    }

    //
    // ----------------------
    // TESTS FOR `find_cycle_in_dependencies`
    // ----------------------
    //

    #[test]
    fn test_find_cycle_in_dependencies_no_cycle() {
        let mut graph = HashMap::new();
        graph.insert("A".to_string(), vec!["B".to_string()]);
        graph.insert("B".to_string(), vec!["C".to_string()]);
        graph.insert("C".to_string(), vec![]);

        let cycle = find_all_cycles_in_dependencies(&graph, 3);
        assert!(cycle.is_empty(), "There should be no cycle in A->B->C");
    }

    #[test]
    fn test_find_cycle_in_dependencies_simple_cycle() {
        let mut graph = HashMap::new();
        graph.insert("A".to_string(), vec!["B".to_string()]);
        graph.insert("B".to_string(), vec!["A".to_string()]);

        let cycle = find_all_cycles_in_dependencies(&graph, 2);
        assert!(!cycle.is_empty(), "A->B->A should be detected as a cycle");
    }

    //
    // ----------------------
    // TESTS FOR `MustNotHaveCircularDependencies`
    // ----------------------
    //

    #[test]
    fn test_rule_no_cycle_ok() {
        let mut graph = HashMap::new();
        graph.insert("A".to_string(), vec!["B".to_string()]);
        graph.insert("B".to_string(), vec!["C".to_string()]);
        graph.insert("C".to_string(), vec![]);

        let project = RustProject { files: vec![] };
        let rule = MustNotHaveCircularDependencies { max_depth: 3 };

        let result = rule.apply(&project);
        assert!(
            result.is_ok(),
            "No error should be triggered for a linear graph"
        );
    }

    #[test]
    fn test_cycle_directly_in_find_cycle() {
        let mut graph = HashMap::new();
        graph.insert("A".to_string(), vec!["B".to_string()]);
        graph.insert("B".to_string(), vec!["A".to_string()]);

        let result = find_all_cycles_in_dependencies(&graph, 1);
        assert!(!result.is_empty(), "A->B->A should be detected as a cycle");
    }

    #[test]
    fn test_rule_empty_graph() {
        let project = RustProject { files: vec![] };

        let rule = MustNotHaveCircularDependencies { max_depth: 2 };
        let result = rule.apply(&project);
        assert!(result.is_ok(), "An empty graph should have no cycles");
    }

    //
    // ----------------------
    // TEST CASES FOR FALSE POSITIVES (Self-References)
    // ----------------------
    //

    /// **Test: Ignore `X -> X` if it has no other dependencies**
    #[test]
    fn test_ignore_trivial_self_reference() {
        let mut graph = HashMap::new();
        graph.insert("X".to_string(), vec!["X".to_string()]);

        let cycles = find_all_cycles_in_dependencies(&graph, 3);

        assert!(
            cycles.is_empty(),
            "A trivial self-reference (`X -> X`) should NOT be reported as a cycle."
        );
    }

    /// **Test: Self-reference inside a larger cycle should still be detected**
    #[test]
    fn test_detect_real_cycle_with_self_reference() {
        let mut graph = HashMap::new();
        graph.insert("X".to_string(), vec!["X".to_string(), "Y".to_string()]);
        graph.insert("Y".to_string(), vec!["X".to_string()]);

        let cycles = find_all_cycles_in_dependencies(&graph, 3);

        assert!(
            !cycles.is_empty(),
            "A real cycle (`Y -> X -> Y`) should be detected even if `X` has a self-reference."
        );

        assert!(
            cycles.iter().any(|c| c.contains("Y -> X -> Y")),
            "The detected cycle should include `Y -> X -> Y`."
        );

        assert!(
            !cycles.iter().any(|c| c == "X -> X"),
            "The trivial self-reference `X -> X` should be ignored."
        );
    }

    /// **Test: Multiple independent self-references should all be ignored**
    #[test]
    fn test_multiple_trivial_self_references() {
        let mut graph = HashMap::new();
        graph.insert("A".to_string(), vec!["A".to_string()]);
        graph.insert("B".to_string(), vec!["B".to_string()]);
        graph.insert("C".to_string(), vec!["C".to_string()]);

        let cycles = find_all_cycles_in_dependencies(&graph, 3);

        assert!(
                cycles.is_empty(),
                "Multiple trivial self-references (`A -> A`, `B -> B`, `C -> C`) should NOT be reported."
            );
    }

    //
    // ----------------------
    // TEST CASES FOR DETECTING REAL CYCLES
    // ----------------------
    //

    /// **Test: Detecting a simple cycle**
    #[test]
    fn test_detect_simple_cycle() {
        let mut graph = HashMap::new();
        graph.insert("A".to_string(), vec!["B".to_string()]);
        graph.insert("B".to_string(), vec!["A".to_string()]);

        let cycles = find_all_cycles_in_dependencies(&graph, 3);

        assert!(
            !cycles.is_empty(),
            "A real cycle (`A -> B -> A`) should be detected."
        );

        assert!(
            cycles.iter().any(|c| c.contains("B -> A -> B")),
            "The detected cycle should include `B -> A -> B`."
        );
    }

    /// **Test: No cycle should be detected in an acyclic graph**
    #[test]
    fn test_no_cycle_in_acyclic_graph() {
        let mut graph = HashMap::new();
        graph.insert("A".to_string(), vec!["B".to_string()]);
        graph.insert("B".to_string(), vec!["C".to_string()]);
        graph.insert("C".to_string(), vec![]);

        let cycles = find_all_cycles_in_dependencies(&graph, 3);

        assert!(
            cycles.is_empty(),
            "No cycles should be detected in an acyclic graph."
        );
    }
}
