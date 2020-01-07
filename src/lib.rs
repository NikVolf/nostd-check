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
}

/// Check of the particular dependency package.
pub struct DependencyCheckSummary {
    /// Package id checked.
    pub package_id: String,
    /// Result of the check.
    pub result: DependencyCheckResult,
}

pub fn check(deps: &[DependencySummary]) -> Vec<DependencyCheckSummary> {
    let mut result = Vec::new();
    for dep in deps.iter() {
        let check_result = match explore(dep.lib_path.clone()) {
            Exploration::NoStd => DependencyCheckResult::Ok,
            Exploration::Conditional =>
                if dep.std_activated {
                    DependencyCheckResult::IsConditionallyNoStdButStdIsActivated
                } else {
                    DependencyCheckResult::Ok
                },
            Exploration::Std => DependencyCheckResult::IsPurelyStd,
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