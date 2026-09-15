//! One place that decides what a path's segments are.
//!
//! Paths reach this crate from two sources:
//!
//! - a hook runner (prek, pre-commit) passes what git reports, which is
//!   repository-relative and `/`-separated on every platform, Windows
//!   included;
//! - tests and callers using [`std::path`] produce native separators, so the
//!   same file can arrive as `…\src\parser.rs` on Windows.
//!
//! Splitting on `/` alone handled the first and not the second, so a native
//! Windows path was one segment and `src` was never found there.
//!
//! Splitting on both characters unconditionally would fix that and break
//! something else: `\` is a legal filename character on Unix, so a file
//! genuinely named `src\core\mover.rs` would be reinterpreted as
//! `src/core/mover.rs` and select a unit-test filter for a module that does
//! not exist.
//!
//! [`std::path::Component`] already draws this line correctly, per platform:
//! on Windows both `\` and `/` separate, on Unix only `/` does. Deferring to
//! it means each platform reads a path the way that platform means it, with no
//! separator guessing here.

use std::path::{Component, Path};

/// Split a path into its meaningful segments, using the platform's own rules.
///
/// Prefixes (`C:`), root markers and `.` are dropped, so `./src/x.rs` and
/// `src/x.rs` agree, and an absolute path contributes only its named parts.
pub fn path_segments(file_path: &str) -> Vec<&str> {
    Path::new(file_path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(segment) => segment.to_str(),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_style_paths_split_on_every_platform() {
        // What a hook runner actually passes, Windows included.
        assert_eq!(
            path_segments("src/analysis/config.rs"),
            ["src", "analysis", "config.rs"]
        );
        assert_eq!(
            path_segments("crates/foo/src/lib.rs"),
            ["crates", "foo", "src", "lib.rs"]
        );
    }

    #[test]
    fn leading_dot_and_empty_segments_are_dropped() {
        assert_eq!(path_segments("./src//x.rs"), ["src", "x.rs"]);
        assert_eq!(path_segments(""), Vec::<&str>::new());
    }

    #[cfg(unix)]
    #[test]
    fn a_backslash_in_a_unix_filename_is_part_of_the_name() {
        // Regression for the review on #11: treating `\` as a separator here
        // would turn one oddly-named file into a three-level module path and
        // select a filter for a module that does not exist.
        assert_eq!(path_segments(r"src\core\mover.rs"), [r"src\core\mover.rs"]);
        assert_eq!(path_segments(r"src/odd\name.rs"), ["src", r"odd\name.rs"]);
    }

    #[cfg(windows)]
    #[test]
    fn native_windows_paths_split_on_backslash() {
        // Regression for the review on #10: these arrive from
        // `TempDir::path().join(..)` and used to collapse to a single segment.
        assert_eq!(
            path_segments(r"C:\Users\runner\Temp\.tmpAbC\src\calculator.rs"),
            ["Users", "runner", "Temp", ".tmpAbC", "src", "calculator.rs"]
        );
        // Windows accepts both separators, so a git-style path still works.
        assert_eq!(
            path_segments(r"crates\foo/src\lib.rs"),
            ["crates", "foo", "src", "lib.rs"]
        );
    }
}
