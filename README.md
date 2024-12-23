# Rust Arkitect - Proof of Concept

**Rust Arkitect** is a Proof of Concept inspired by [phparkitect/arkitect](https://github.com/phparkitect/arkitect), designed to define and validate architectural rules in Rust projects.

## Goal

Provide a tool to:
- Define components and dependency rules
- Validate that the codebase adheres to these rules

## Example
Given a project with the following structure:

```plaintext
src/
├── application/
│   ├── mod.rs
│   └── service.rs
├── domain/
│   ├── mod.rs
│   └── entity.rs
├── infrastructure/
│   ├── mod.rs
│   └── database.rs
```

You can define architectural rules:

```rust
pub fn define_architecture() -> Rules {
    Architecture::with_components()
        .component(Components::Application).defined_by("./src/application")
        .component(Components::Domain).defined_by("./src/domain")
        .component(Components::Infrastructure).defined_by("./src/infrastructure")
        .rules_for(Components::Domain).must_not_depend_on_anything()
        .rules_for(Components::Application).depends_on(&[Components::Domain])
        .rules_for(Components::Infrastructure).depends_on(&[Components::Domain, Components::Application])
        .build()
}
```

Then you can validate them:
```rust
#[test]
fn test_architecture_rules() {
let architecture = sample_project::dependency_rules::define_architecture();

    let result = architecture.validate();

    assert_eq!(result, Ok(()))
}
```

## Project Status
- Only rule DSL is implemented
- Validation logic for analyzing real Rust code is still under development

## Feedback

This is a POC, and your feedback is essential!
If you have ideas, suggestions, or would like to contribute, open an issue or submit a pull request.