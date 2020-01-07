mod source;
mod meta;

#[cfg(test)]
mod tests;

pub use crate::source::{explore, Exploration};
pub use crate::meta::{find_all_root_libs, DependencySummary};

/// Result of the dependency check.
#[derive(Debug, PartialEq)]
pub enum DependencyCheckResult {
    /// Dependency is fine.
    Ok,
    /// Dependency is not fine and cannot be (is std).
    IsPurelyStd,
    /// Dependenct is not fine, but can be (if std feature is deactivated).
    IsConditionallyNoStdButStdIsActivated,
    /// Unknown,
    Unknown,
}

/// Check of the particular dependency package.
pub struct DependencyCheckSummary {
    /// Package id checked.
    pub package_id: String,
    /// Result of the check.
    pub result: DependencyCheckResult,
}

/// Checks current directory
pub fn check() -> Vec<DependencyCheckSummary> {
    let deps = find_all_root_libs();
    check_dependencies(&deps)
}

/// Checks some dependencies (can be queried by `find_all_root_libs`)
pub fn check_dependencies(deps: &[DependencySummary]) -> Vec<DependencyCheckSummary> {
    let mut result = Vec::new();
    for dep in deps.iter() {
        println!("Checking {}: at {}", dep.package_id, dep.lib_path.to_string_lossy());
        let check_result = match explore(dep.lib_path.clone()) {
            Exploration::NoStd => DependencyCheckResult::Ok,
            Exploration::Conditional =>
                if dep.std_activated {
                    DependencyCheckResult::IsConditionallyNoStdButStdIsActivated
                } else {
                    DependencyCheckResult::Ok
                },
            Exploration::Std => DependencyCheckResult::IsPurelyStd,
            Exploration::SkippedDueToParseError => DependencyCheckResult::Unknown,
        };

        result.push(
            DependencyCheckSummary {
                package_id: dep.package_id.to_string(),
                result: check_result,
            }
        )
    }
    result
}