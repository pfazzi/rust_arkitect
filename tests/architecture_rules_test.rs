mod sample_project;

#[test]
fn test_architecture_rules() {
    let architecture = sample_project::dependency_rules::define_architecture();

    let result = architecture.validate();

    assert_eq!(result, Ok(()))
}