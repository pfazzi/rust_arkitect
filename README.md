# Rust Arkitect - Proof of Concept

**Rust Arkitect** is a Proof of Concept inspired by [phparkitect/arkitect](https://github.com/phparkitect/arkitect), designed to define and validate architectural rules in Rust projects. By leveraging a simple, developer-friendly DSL, Rust Arkitect helps maintain clean architectures.

### Why It Matters
Architectural rules are essential for maintaining clean, modular, and scalable codebases. Rust Arkitect provides developers with the tools to:
- Clearly document architectural rules
- Consistently enforce rules across the codebase, preventing accidental violations
- Catch architectural issues early with immediate feedback during testing

By integrating directly with Rust’s testing framework, Rust Arkitect allows teams to ensure their architecture evolves safely as their codebase grows.

### Readable and Expressive
Rust Arkitect provides a developer-friendly DSL that simplifies defining and enforcing architectural rules.
The DSL is designed to be as close to plain English as possible, making it easy to understand even for those new to the project. For example:
```rust
let project = Project::load("./../rust_arkitect/sample_project/src");

let rules = ArchitecturalRules::define()
    .component("Domain")
        .located_at("crate::domain")
        .must_not_depend_on_anything()
    .component("Application")
        .located_at("crate::application")
        .may_depend_on(&["Domain"])
    .finalize();
```
The DSL mirrors how developers naturally think about architecture, making it both clear and concise.

### Test-Driven Validation
The DSL integrates seamlessly with Rust’s testing framework, allowing you to assert compliance as part of your test suite:

```rust
let result = Arkitect::ensure_that(project).complies_with(rules);

assert!(result.is_ok());
```

## Example
Given a project with the following structure:

```plaintext
src/
├── application/
│   ├── mod.rs
│   └── service.rs
├── domain/
│   ├── mod.rs
│   ├── service.rs
│   └── entity.rs
├── infrastructure/
│   ├── mod.rs
│   ├── auth.rs
│   └── database.rs
```

You can define and test architectural rules:
```rust
#[test]
fn test_architectural_rules() {
    let project = Project::load("./../rust_arkitect/sample_project/src");

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .component("Application")
            .located_at("crate::application")
            .may_depend_on(&["Domain"])

        .component("Domain")
            .located_at("crate::domain")
            .must_not_depend_on_anything()

        .component("Infrastructure")
            .located_at("crate::infrastructure")
            .may_depend_on(&["Domain", "Application"])

        .finalize();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(result.is_ok());
}
```

## Logging Support

Rust Arkitect includes logging support to provide detailed information during the validation process. This feature allows you to toggle between verbose and simple output by initializing the logger using Arkitect::init_logger().

### How to Enable Logging

To enable logging, simply call Arkitect::init_logger() at the start of your tests or application. For example:
```rust
use rust_arkitect::dsl::{ArchitecturalRules, Arkitect, Project};

#[test]
fn test_logging_in_architecture_rules() {
    // Initialize logging
    Arkitect::init_logger();

    let project = Project::load("./../rust_arkitect/sample_project/src");

    let rules = ArchitecturalRules::define()
        .component("Application")
            .located_at("crate::application")
            .may_depend_on(&["Domain"])

        .component("Domain")
            .located_at("crate::domain")
            .must_not_depend_on_anything()
        .finalize();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(result.is_ok());
}
```

### Controlling Verbosity

You can adjust the verbosity of the logging output by setting the RUST_LOG environment variable:
- Verbose Mode: Shows detailed information, including applied and respected rules:
```bash
RUST_LOG=info cargo test -- --nocapture
```
Example Output:
```plaintext
WARN  rust_arkitect > ❌ Rule violated: crate::application may depend on [crate::domain]
```

## Project Status

- **Implemented**:
    - DSL for defining architectural rules
    - Validation logic works on the example project provided in this repository

- **Pending**:
    - Validation on real-world Rust projects
    - Support for more complex architectural patterns

This project is in the early stages and serves as a demonstration of the core concept.

## Feedback

Rust Arkitect is an early Proof of Concept, and your feedback is invaluable to its growth.
If you have ideas, suggestions, or would like to contribute, open an issue or submit a pull request.