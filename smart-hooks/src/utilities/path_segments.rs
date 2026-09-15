//! One place that decides what a path's segments are.
//!
//! Paths reach this crate from two sources that disagree about separators:
//!
//! - a hook runner (prek, pre-commit) passes repository-relative POSIX paths,
//!   `src/parser.rs`, because that is what git reports;
//! - callers and tests using [`std::path`] produce native separators, so on
//!   Windows the same file arrives as `…\src\parser.rs`.
//!
//! Splitting on `/` alone saw a Windows path as a single segment, so `src` was
//! never found and every path-based decision silently came out negative there.
//! Both separators are therefore accepted on every platform, so that a given
//! path string classifies identically wherever the code runs.
//!
//! The cost is that a Unix file whose *name* contains a literal backslash is
//! split at it. That is legal on Unix and pathological in a source tree; making
//! the behaviour platform-dependent instead would mean the same input
//! classified differently on different machines, which is worse for a tool that
//! decides which tests to skip.

/// Split a path into its meaningful segments, accepting either separator.
///
/// Empty segments and `.` are dropped, so `./src//x.rs` and `src/x.rs` agree.
pub fn path_segments(file_path: &str) -> Vec<&str> {
    file_path
        .split(['/', '\\'])
        .filter(|s| !s.is_empty() && *s != ".")
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posix_paths_split_on_slash() {
        assert_eq!(
            path_segments("src/analysis/config.rs"),
            ["src", "analysis", "config.rs"]
        );
    }

    #[test]
    fn windows_paths_split_on_backslash() {
        assert_eq!(
            path_segments(r"C:\Users\runner\Temp\.tmpAbC\src\calculator.rs"),
            [
                "C:",
                "Users",
                "runner",
                "Temp",
                ".tmpAbC",
                "src",
                "calculator.rs"
            ]
        );
    }

    #[test]
    fn mixed_separators_are_accepted() {
        assert_eq!(
            path_segments(r"crates\foo/src\lib.rs"),
            ["crates", "foo", "src", "lib.rs"]
        );
    }

    #[test]
    fn empty_and_dot_segments_are_dropped() {
        assert_eq!(path_segments("./src//x.rs"), ["src", "x.rs"]);
        assert_eq!(path_segments(r".\src\x.rs"), ["src", "x.rs"]);
        assert_eq!(path_segments(""), Vec::<&str>::new());
    }
}
