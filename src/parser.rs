use syn::{visit::Visit, ItemUse, UseTree};
use std::fs;
use std::path::Path;

#[derive(Default)]
struct DependencyVisitor {
    dependencies: Vec<String>,
}

impl<'ast> Visit<'ast> for DependencyVisitor {
    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        match &node.tree {
            UseTree::Path(path) => self.collect_use_tree(&path.tree, path.ident.to_string()),
            _ => (),
        }
    }
}

impl DependencyVisitor {
    fn collect_use_tree(&mut self, tree: &UseTree, prefix: String) {
        match tree {
            UseTree::Path(path) => {
                let new_prefix = if prefix.is_empty() {
                    path.ident.to_string()
                } else {
                    format!("{}::{}", prefix, path.ident)
                };
                self.collect_use_tree(&path.tree, new_prefix);
            }
            UseTree::Group(group) => {
                for item in &group.items {
                    self.collect_use_tree(item, prefix.clone());
                }
            }
            UseTree::Name(name) => {
                self.dependencies.push(format!("{}::{}", prefix, name.ident));
            }
            _ => (),
        }
    }
}

pub fn parse_source(file_path: &Path) -> Vec<String> {
    let content = fs::read_to_string(file_path).expect("Failed to read file");
    let syntax: syn::File = syn::parse_file(&content).expect("Failed to parse Rust file");

    let mut visitor = DependencyVisitor::default();
    visitor.visit_file(&syntax);

    visitor.dependencies
}