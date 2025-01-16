# 📐 Rust Arkitect

[![codecov](https://codecov.io/github/pfazzi/rust_arkitect/graph/badge.svg?token=FVLITXKTQE)](https://codecov.io/github/pfazzi/rust_arkitect)
[![Crate Version](https://img.shields.io/crates/v/rust_arkitect.svg)](https://crates.io/crates/rust_arkitect)
[![Crate Downloads](https://img.shields.io/crates/d/rust_arkitect.svg)](https://crates.io/crates/rust_arkitect)
[![Crate License](https://img.shields.io/crates/l/rust_arkitect.svg)](https://crates.io/crates/rust_arkitect)

**Rust Arkitect** is a powerful tool that puts your architectural rules to the test, ensuring your Rust projects maintain a clean and modular structure. Inspired by [phparkitect/arkitect](https://github.com/phparkitect/arkitect), it provides a developer-friendly DSL to define and validate architectural constraints seamlessly integrated into your workflow.

# 🚀 Why Rust Arkitect?
Rust Arkitect helps you:
- **Define Rules Clearly:** Use a simple DSL to specify architectural rules that mirror natural language
- **Refactor Legacy Code Safely:** Establish a baseline of violations, monitor improvements, and prevent regressions
- **Validate Continuously:** Integrate architectural tests seamlessly into your test suite, with immediate feedback during development

# 🧑‍💻 Getting Started
Add Rust Arkitect to your `Cargo.toml`:
```toml
[dev-dependencies]
rust_arkitect = "0.1"
```
Define your architectural rules:
```rust
use rust_arkitect::dsl::{ArchitecturalRules, Arkitect, Project};

#[test]
fn test_architectural_rules() {
    let project = Project::from_relative_path(file!(), "./../src");

    let rules = ArchitecturalRules::define()
        .component("Domain")
            .located_at("crate::domain")
            .allow_external_dependencies(&["std::fmt"])
            .must_not_depend_on_anything()
        .component("Application")
            .located_at("crate::application")
            .may_depend_on(&["Domain"])
        .finalize();

    let result = Arkitect::ensure_that(project).complies_with(rules);

    assert!(result.is_ok(), "Detected {} violations", result.err().unwrap().len());
}

```

#  🏗️ Refactoring Legacy Code with Rust Arkitect

Rust Arkitect enables structured refactoring of legacy codebases. By establishing a baseline of current architectural violations, you can track improvements over time and ensure that no new violations are introduced during refactoring.

## Example
Rust Arkitect enables structured refactoring of legacy codebases. By establishing a baseline of current architectural violations, you can track improvements over time and ensure that no new violations are introduced during refactoring.

Given a project with the following structure:
```text
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
fn test_architecture_baseline() {
    let project = Project::from_relative_path(file!(), "./../src");

    let rules = ArchitecturalRules::define()
        .component("Application")
            .located_at("crate::application")
            .may_depend_on(&["Domain"])

        .component("Domain")
            .located_at("crate::domain")
            .allow_external_dependencies(&["std::fmt"])
            .must_not_depend_on_anything()

        .component("Infrastructure")
            .located_at("crate::infrastructure")
            .allow_external_dependencies(&["serde"])
            .may_depend_on(&["Domain", "Application"])

        .finalize();

    let baseline_violations = 30;
    let result = Arkitect::ensure_that(project).with_baseline(baseline_violations).complies_with(rules);

    assert!(
        result.is_ok(),
        "Violations increased! Expected at most {}, found {}.",
        baseline_violations,
        current_violations
    );
}
```
This test ensures that the number of violations does not exceed the established baseline, promoting continuous improvement in your codebase's architecture.

# 🔍 Logging Violations

Rust Arkitect includes logging support to provide detailed information during the validation process.
To enable logging, simply call `Arkitect::init_logger()` at the start of your tests. For example:
```rust
use rust_arkitect::dsl::{ArchitecturalRules, Arkitect, Project};

#[test]
fn test_logging_in_architecture_rules() {
    // Initialize logging
    Arkitect::init_logger();

    ...

    assert!(result.is_ok());
}
```

You can adjust the verbosity of the logging output by setting the RUST_LOG environment variable:
```bash
RUST_LOG=error cargo test -- --nocapture
```
Example Output:
```plaintext
[2024-12-30T12:17:08Z ERROR rust_arkitect::dsl] 🟥 Rule crate::event_sourcing may depend on [std::fmt] violated: forbidden dependencies to [crate::domain::events::event] in file:///users/random/projects/acme_project/src/event_sourcing/events.rs
[2024-12-30T12:17:08Z ERROR rust_arkitect::dsl] 🟥 Rule crate::utils may not depend on any modules violated: forbidden dependencies to [crate::infrastructure::redis::*] in file:///users/random/projects/acme_project/src/utils/refill.rs
```

# 😇 Built with Its Own Rules

Rust Arkitect is built and tested using the same architectural rules it enforces. This ensures the tool remains consistent with the principles it promotes. You can explore the [architecture tests here](tests/test_architecture.rs) to see it in action.

# 🛠️ Contribute

Rust Arkitect is an evolving project, and your feedback is invaluable. Whether you have suggestions, encounter issues, or wish to contribute, please open an issue or submit a pull request. Together, we can build robust and maintainable Rust applications.
