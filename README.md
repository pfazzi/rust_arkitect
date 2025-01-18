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
rust_arkitect = "0.2"
```
Define your architectural rules:
```rust
use rust_arkitect::dsl::{ArchitecturalRules, Arkitect, Project};

#[test]
fn test_architectural_rules() {
    let project = Project::from_current_crate();

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .rules_for_module("sample_project::application")
            .may_depend_on(&["sample_project::domain"])

        .rules_for_module("sample_project::domain")
            .must_not_depend_on_anything()

        .rules_for_module("sample_project::infrastructure")
            .may_depend_on(&["sample_project::domain", "sample_project::application"])

        .build();

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
    let project = Project::from_current_workspace();

    #[rustfmt::skip]
    let rules = ArchitecturalRules::define()
        .rules_for_crate("application")
            .may_depend_on(&["domain"])

        .rules_for_module("domain")
            .must_not_depend_on_anything()

        .rules_for_module("infrastructure")
            .may_depend_on(&["domain", "application"])

        .build();

    let baseline_violations = 30;
    let result = Arkitect::ensure_that(project)
        .with_baseline(baseline_violations)
        .complies_with(rules);

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
[2024-12-30T12:17:08Z ERROR rust_arkitect::dsl] 🟥 Rule my_project::event_sourcing may depend on [std::fmt] violated: forbidden dependencies to [my_project::domain::events::event] in file:///users/random/projects/acme_project/src/event_sourcing/events.rs
[2024-12-30T12:17:08Z ERROR rust_arkitect::dsl] 🟥 Rule my_project::utils may not depend on any modules violated: forbidden dependencies to [my_project::infrastructure::redis::*] in file:///users/random/projects/acme_project/src/utils/refill.rs
```

# 🧙‍♂️ Custom Rules
Rust Arkitect allows you to create custom rules to test your project's architecture. These rules can be implemented by creating a struct and implementing the `Rule` trait for it. Below is an example of how to define and use a custom rule in a test:

```rust
// Define a custom rule
struct TestRule;

impl TestRule {
    fn new(_subject: &str, _dependencies: &[&str; 1]) -> TestRule {
        Self {}
    }
}

// Implement Display for the rule for better readability in logs
impl Display for TestRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestRule applied")
    }
}

// Implement the Rule trait
impl Rule for TestRule {
    fn apply(&self, _file: &str) -> Result<(), String> {
        Ok(())
    }

    fn is_applicable(&self, _file: &str) -> bool {
        true
    }
}

#[test]
fn test_custom_rule_execution() {
    // Define the project path
    let project = Project::new();

    // Create a new instance of the custom rule
    let rule = Box::new(TestRule::new("my_crate", &["another_crate::a_module"]));

    // Apply the rule to the project
    let result = Arkitect::ensure_that(project).complies_with(vec![rule]);

    // Assert that the rule passed
    assert!(result.is_ok());
}
```
# 😇 Built with Its Own Rules

Rust Arkitect is built and tested using the same architectural rules it enforces. This ensures the tool remains consistent with the principles it promotes. You can explore the [architecture tests here](tests/test_architecture.rs) to see it in action.

# 🛠️ Contribute

Rust Arkitect is an evolving project, and your feedback is invaluable. Whether you have suggestions, encounter issues, or wish to contribute, please open an issue or submit a pull request. Together, we can build robust and maintainable Rust applications.
