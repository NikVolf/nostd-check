use std::path::PathBuf;
use cargo::util::{
    Config,
    important_paths::find_root_manifest_for_wd
};
use cargo::core::{Workspace, PackageId};

#[derive(Debug, Clone)]
pub struct DependencySummary {
    pub package_id: PackageId,
    pub lib_path: PathBuf,
    pub std_activated: bool,
}

pub fn find_all_root_libs() -> Vec<DependencySummary> {
    let config = Config::default().expect("Cannot infer default config");
    let root = find_root_manifest_for_wd(&config.cwd()).expect("Failed to get root - not a workspace directory?");
    let mut result = Vec::new();

    let workspace = Workspace::new(&root, &config).expect("Failed to get workspace - not a workspace directory?");
    let (package_set, resolve) = cargo::ops::resolve_ws(&workspace).expect("cannot resolve");

    for package_id in package_set.package_ids() {
        let package = package_set.get_one(package_id).expect("Failed to resolve package_id");

        let manifest_root = package.root();
        for target in package.manifest().targets() {
            if target.is_lib() {
                if let cargo::core::manifest::TargetSourcePath::Path(lib_path) = target.src_path() {
                    let mut check_path: PathBuf = manifest_root.clone().into();
                    check_path.push(lib_path.clone());
                    result.push(DependencySummary {
                        package_id: package_id.clone(),
                        lib_path: check_path,
                        std_activated: resolve.features(package_id).contains("std"),
                    });
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {

    #[test]
    fn libs() {
        let libs = super::find_all_root_libs();
        assert_eq!(libs.len(), 128);
    }
}