//! Convert file paths to Rust module names.

/// Split a path into `/`-separated segments, ignoring `./` and empty segments.
///
/// Paths reach us from a hook runner as repository-relative strings
/// (`src/calculator.rs`), so segment matching has to work with or without a
/// leading component.
fn segments(file_path: &str) -> Vec<&str> {
    file_path
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect()
}

/// Index just past the last `src` segment, if the path has one.
fn after_src(segs: &[&str]) -> Option<usize> {
    segs.iter().rposition(|s| *s == "src").map(|i| i + 1)
}

/// Extract the Rust module path for a source file, e.g.
/// `src/analysis/config.rs` -> `analysis::config`.
///
/// Returns `None` for paths outside a `src` directory, for anything under a
/// `tests` directory, for non-Rust files, and for crate roots (`main.rs`,
/// `lib.rs`), which have no module of their own.
pub fn extract_module_name(file_path: &str) -> Option<String> {
    let segs = segments(file_path);

    // A `tests` segment anywhere means test code, not a module of this crate:
    // `src/tests/helpers.rs` is test support, and `tests/src/fixture.rs` is a
    // fixture crate. Selecting a unit-test filter from either is wrong.
    if segs.contains(&"tests") {
        return None;
    }

    let start = after_src(&segs)?;
    let rest = &segs[start..];

    let (last, parents) = rest.split_last()?;
    let stem = last.strip_suffix(".rs")?;

    if parents.is_empty() && (stem == "main" || stem == "lib") {
        return None;
    }

    let mut parts: Vec<&str> = parents.to_vec();
    if stem != "mod" {
        parts.push(stem);
    }

    if parts.is_empty() {
        return None;
    }

    Some(parts.join("::"))
}

/// Whether the path looks like a core module worth integration coverage.
pub fn is_core_module(file_path: &str) -> bool {
    let segs = segments(file_path);
    segs.contains(&"core")
        || segs
            .last()
            .is_some_and(|last| matches!(*last, "types.rs" | "error.rs" | "lib.rs" | "main.rs"))
}

/// Whether the path looks like behavioural logic worth scenario coverage.
pub fn is_behavioral_module(file_path: &str) -> bool {
    segments(file_path)
        .iter()
        .any(|s| matches!(*s, "apply" | "strategy" | "fast_export"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_module_names_from_repository_relative_paths() {
        // This is the shape a hook runner actually passes; see issue #7.
        assert_eq!(
            extract_module_name("src/calculator.rs"),
            Some("calculator".to_string())
        );
        assert_eq!(
            extract_module_name("src/analysis/config.rs"),
            Some("analysis::config".to_string())
        );
    }

    #[test]
    fn extracts_module_names_from_nested_crate_paths() {
        assert_eq!(
            extract_module_name("any-crate/src/core/mover.rs"),
            Some("core::mover".to_string())
        );
        assert_eq!(
            extract_module_name("crates/git-mvh/src/types.rs"),
            Some("types".to_string())
        );
        assert_eq!(
            extract_module_name("./src/analysis/config.rs"),
            Some("analysis::config".to_string())
        );
    }

    #[test]
    fn a_module_whose_name_starts_with_mod_is_not_mangled() {
        // The previous implementation did `.replace("::mod", "")`, turning
        // `utilities::module_utils` into `utilitiesule_utils` — a filter that
        // matches no test.
        assert_eq!(
            extract_module_name("src/utilities/module_utils.rs"),
            Some("utilities::module_utils".to_string())
        );
        assert_eq!(
            extract_module_name("src/models.rs"),
            Some("models".to_string())
        );
    }

    #[test]
    fn maps_mod_rs_to_its_directory() {
        assert_eq!(
            extract_module_name("src/analysis/mod.rs"),
            Some("analysis".to_string())
        );
        assert_eq!(
            extract_module_name("src/a/b/mod.rs"),
            Some("a::b".to_string())
        );
    }

    #[test]
    fn a_directory_named_src_deeper_in_the_tree_wins() {
        assert_eq!(
            extract_module_name("src/vendor/thing/src/inner.rs"),
            Some("inner".to_string())
        );
    }

    #[test]
    fn test_trees_yield_no_module_wherever_src_appears() {
        // Regression for the review on #8: `tests/src/...` is a fixture crate,
        // not this crate's source, and selecting `example` from it is wrong.
        assert_eq!(extract_module_name("tests/src/example.rs"), None);
        assert_eq!(extract_module_name("tests/fixtures/demo/src/lib.rs"), None);
        assert_eq!(extract_module_name("src/tests/helpers.rs"), None);
        assert_eq!(extract_module_name("crates/foo/tests/src/helper.rs"), None);
    }

    #[test]
    fn crate_roots_and_non_rust_paths_have_no_module() {
        assert_eq!(extract_module_name("src/main.rs"), None);
        assert_eq!(extract_module_name("src/lib.rs"), None);
        assert_eq!(extract_module_name("any-crate/src/main.rs"), None);
        assert_eq!(extract_module_name("other/file.rs"), None);
        assert_eq!(extract_module_name("no-src-dir/file.rs"), None);
        assert_eq!(extract_module_name("src/notes.md"), None);
        assert_eq!(extract_module_name("src/mod.rs"), None);
    }

    #[test]
    fn a_nested_main_rs_is_a_module_not_a_crate_root() {
        assert_eq!(
            extract_module_name("src/bin/main.rs"),
            Some("bin::main".to_string())
        );
    }

    #[test]
    fn identifies_core_modules() {
        assert!(is_core_module("src/core/mover.rs"));
        assert!(is_core_module("any-path/src/core/mover.rs"));
        assert!(is_core_module("project/src/types.rs"));
        assert!(is_core_module("crate/src/error.rs"));
        assert!(is_core_module("some/src/lib.rs"));
        assert!(!is_core_module("project/src/apply/strategy.rs"));
    }

    #[test]
    fn core_module_matching_is_segment_aware() {
        // `corestore` is not `core`, and `my_types.rs` is not `types.rs`.
        assert!(!is_core_module("src/corestore/thing.rs"));
        assert!(!is_core_module("src/my_types.rs"));
    }

    #[test]
    fn identifies_behavioural_modules() {
        assert!(is_behavioral_module("src/apply/strategy.rs"));
        assert!(is_behavioral_module("project/src/strategy/adaptive.rs"));
        assert!(is_behavioral_module("crate/src/fast_export/parser.rs"));
        assert!(!is_behavioral_module("project/src/core/mover.rs"));
    }
}
