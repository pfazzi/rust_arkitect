use std::path::Path;
use rust_arkitect::parser::parse_source;

#[test]
fn test_parse_source() {
    let dependencies = parse_source(Path::new("conversion/application.rs"));

    assert_eq!(
        dependencies,
        vec![
            "rust_arkitect::sample_project::domain".to_string(),
            "rust_arkitect::sample_project::infrastructure".to_string()
        ]
    );
}
