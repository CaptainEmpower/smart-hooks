use std::path::Path;

/// Impact level analysis for determining test requirements
/// Focused on categorizing changes by their potential impact

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImpactLevel {
    None,
    Low,
    Medium,
    High,
}

impl ImpactLevel {
    /// Return the higher of two impact levels
    pub fn max(self, other: Self) -> Self {
        if self >= other {
            self
        } else {
            other
        }
    }
}

impl std::fmt::Display for ImpactLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImpactLevel::None => write!(f, "none"),
            ImpactLevel::Low => write!(f, "low"),
            ImpactLevel::Medium => write!(f, "medium"),
            ImpactLevel::High => write!(f, "high"),
        }
    }
}

/// Determine impact level based on changed files using robust pattern matching
pub fn determine_impact_level(changed_files: &[String]) -> ImpactLevel {
    // Critical files that always trigger high impact
    let critical_files = &["lib.rs", "main.rs", "error.rs", "types.rs"];

    // High impact modules (core functionality)
    let high_impact_modules = &["core/mod.rs", "apply/mod.rs", "strategy/mod.rs"];

    // Medium impact prefixes
    let medium_impact_prefixes = &["core/", "apply/", "fast_export/"];

    // Low impact prefixes
    let low_impact_prefixes = &["path/", "repository/", "history/"];

    // Dependency changes always require integration tests
    let dependency_files = &["Cargo.toml", "crates/git-mvh/Cargo.toml"];

    let mut max_impact = ImpactLevel::None;

    for file in changed_files {
        let file_path = Path::new(file);

        // Check for dependency changes (always high impact)
        if dependency_files.iter().any(|&dep| file.ends_with(dep)) {
            return ImpactLevel::High;
        }

        // Extract the relative path from git-mvh src directory
        let relative_file = if let Ok(stripped) = file_path.strip_prefix("crates/git-mvh/src") {
            stripped.to_string_lossy().replace('\\', "/")
        } else {
            file.clone()
        };

        // Check critical files first
        if let Some(filename) = file_path.file_name() {
            if critical_files.iter().any(|&f| filename == f) {
                return ImpactLevel::High;
            }
        }

        // Check high impact modules (exact matches)
        if high_impact_modules
            .iter()
            .any(|&pattern| relative_file == pattern)
        {
            return ImpactLevel::High;
        }

        // Check medium impact prefixes
        if medium_impact_prefixes
            .iter()
            .any(|&prefix| relative_file.starts_with(prefix))
        {
            max_impact = max_impact.max(ImpactLevel::Medium);
            continue;
        }

        // Check low impact prefixes
        if low_impact_prefixes
            .iter()
            .any(|&prefix| relative_file.starts_with(prefix))
        {
            max_impact = max_impact.max(ImpactLevel::Low);
            continue;
        }

        // Any other .rs file in src is at least low impact
        if relative_file.ends_with(".rs") {
            max_impact = max_impact.max(ImpactLevel::Low);
        }
    }

    max_impact
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impact_level_ordering() {
        assert!(ImpactLevel::High > ImpactLevel::Medium);
        assert!(ImpactLevel::Medium > ImpactLevel::Low);
        assert!(ImpactLevel::Low > ImpactLevel::None);
    }

    #[test]
    fn test_impact_level_max() {
        assert_eq!(ImpactLevel::Low.max(ImpactLevel::High), ImpactLevel::High);
        assert_eq!(ImpactLevel::High.max(ImpactLevel::Low), ImpactLevel::High);
        assert_eq!(
            ImpactLevel::Medium.max(ImpactLevel::Medium),
            ImpactLevel::Medium
        );
    }

    #[test]
    fn test_determine_impact_level_high() {
        let files = vec!["crates/git-mvh/src/types.rs".to_string()];
        assert_eq!(determine_impact_level(&files), ImpactLevel::High);
    }

    #[test]
    fn test_determine_impact_level_dependency() {
        let files = vec!["Cargo.toml".to_string()];
        assert_eq!(determine_impact_level(&files), ImpactLevel::High);
    }

    #[test]
    fn test_determine_impact_level_medium() {
        let files = vec!["crates/git-mvh/src/core/mover.rs".to_string()];
        assert_eq!(determine_impact_level(&files), ImpactLevel::Medium);
    }

    #[test]
    fn test_determine_impact_level_none() {
        let files = vec!["README.md".to_string()];
        assert_eq!(determine_impact_level(&files), ImpactLevel::None);
    }
}
