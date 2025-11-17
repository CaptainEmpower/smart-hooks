/// Test fixtures and utilities for smart-hooks testing

pub fn sample_rust_files() -> Vec<String> {
    vec![
        "crates/git-mvh/src/core/move_validator.rs".to_string(),
        "crates/git-mvh/src/core/history_processor.rs".to_string(),
        "crates/git-mvh/src/apply/strategy.rs".to_string(),
        "crates/git-mvh/src/types.rs".to_string(),
    ]
}

pub fn sample_mixed_files() -> Vec<String> {
    vec![
        "crates/git-mvh/src/core/mover.rs".to_string(),
        "README.md".to_string(),
        "Cargo.toml".to_string(),
        "crates/git-mvh/src/strategy/adaptive.rs".to_string(),
    ]
}

pub fn sample_non_rust_files() -> Vec<String> {
    vec![
        "README.md".to_string(),
        "Cargo.toml".to_string(),
        "docs/guide.md".to_string(),
        ".gitignore".to_string(),
    ]
}