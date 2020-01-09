use std::path::PathBuf;
use cargo::util::{
    Config,
    important_paths::find_root_manifest_for_wd
};
use cargo::core::{Workspace, PackageId};
use cargo::core::resolver::ResolveOpts;
use std::collections::HashSet;
use toml::{self, Value};

#[derive(Debug, Clone)]
pub struct DependencySummary {
    pub package_id: PackageId,
    pub lib_path: PathBuf,
    pub std_activated: bool,
}

pub fn toml_from_file<P: AsRef<std::path::Path>>(p: P) -> std::io::Result<Value> {
    use std::io::Read;

    let mut f = std::fs::File::open(p.as_ref())?;

    let mut s = String::new();
    f.read_to_string(&mut s)?;

    let toml: Value = toml::from_str(&s)?;
    Ok(toml)
}

pub fn is_proc_macro(mut root_path: PathBuf) -> bool {
    root_path.push("Cargo.toml");
    let manifest_toml = toml_from_file(root_path.clone()).expect("failed to load toml");

    if let Some(table) = manifest_toml.get("lib") {
        if let Some(Value::Boolean(true)) = table.get("proc-macro") {
            return true;
        }
    }

    false
}

pub fn push_reg_deps(
    data: &mut HashSet<String>,
    proc_macro_set: &HashSet<String>,
    ban_list: &mut HashSet<String>,
    mut root_path: PathBuf,
) -> i32 {
    root_path.push("Cargo.toml");
    let manifest_toml = toml_from_file(root_path.clone()).expect("failed to load toml");
    let mut result = 0;

    if let Some(table) = manifest_toml.get("dependencies") {
        if let Some(table) = table.as_table() {
            for (mut dep_name, dep_table) in table.iter() {
                if let Some(&Value::String(ref name)) = dep_table.get("package") {
                    dep_name = name;
                }

                if let Some(Value::Boolean(true)) = dep_table.get("optional") {
                    continue;
                }

                if let Some(feature_table) = manifest_toml.get("features") {
                    if let Some(&Value::Array(ref array)) = feature_table.get("std") {
                        if array.iter().find(|v|
                            if let Value::String(f) = v {
                                f == dep_name
                            } else { false }
                        ).is_some() {
                            ban_list.insert(dep_name.to_string());
                            continue;
                        }
                    }
                }

                if proc_macro_set.contains(dep_name) { continue; }
                if !data.contains(dep_name) {
                    result += 1;
                    data.insert(dep_name.to_string());
                }
            }
        }
    }
    result
}

pub fn find_all_root_libs() -> Vec<DependencySummary> {
    let config = Config::default().expect("Cannot infer default config");
    let root = find_root_manifest_for_wd(&config.cwd()).expect("Failed to get root - not a workspace directory?");
    let mut result = Vec::new();

    let resolve_opts = ResolveOpts {
        uses_default_features: false,
        dev_deps: false,
        all_features: false,
        features: Default::default(),
    };

    let workspace = Workspace::new(&root, &config).expect("Failed to get workspace - not a workspace directory?");

    let mut regular_deps = HashSet::new();

    let mut proc_macro_set = HashSet::<String>::new();
    let mut ban_list = HashSet::new();
    ban_list.insert("serde".to_string());
    ban_list.insert("log".to_string());
    ban_list.insert("nodrop".to_string());

    let mut pkg_specs = Vec::new();
    for member in workspace.members() {
        pkg_specs.push(cargo::core::PackageIdSpec::from_package_id(member.manifest().summary().package_id()));
        if is_proc_macro(member.root().into()) {
            continue;
        }
        regular_deps.insert(member.name().to_string());

        push_reg_deps(&mut regular_deps, &proc_macro_set, &mut ban_list, member.root().into());
    }

    let resolve_result = cargo::ops::resolve_ws_with_opts(&workspace, resolve_opts, &pkg_specs)
        .expect("cannot resolve");

    let package_set = resolve_result.pkg_set;
    let resolve = resolve_result.targeted_resolve;

    for package_id in package_set.package_ids() {
        let package = package_set.get_one(package_id).expect("Failed to resolve package_id");
        if is_proc_macro(package.root().into()) {
            proc_macro_set.insert(package.name().to_string());
        }
    }

    // drying out resolved packages until no more regular dependencies can be added
    // TODO: probably more optimal way exists to do it.
    loop {
        let mut found = 0;
        for package_id in package_set.package_ids() {
            let package = package_set.get_one(package_id).expect("Failed to resolve package_id");

            if regular_deps.contains(&package.name().to_string()) {
                found += push_reg_deps(&mut regular_deps, &proc_macro_set, &mut ban_list, package.root().into());
            }
        }

        if found == 0 { break; }
    }

    for ban_entry in ban_list.drain() {
        regular_deps.remove(&ban_entry);
    }

    for package_id in package_set.package_ids() {
        let package = package_set.get_one(package_id).expect("Failed to resolve package_id");
        let manifest_root = package.root();

        // skipping build deps
        if !regular_deps.contains(&package.name().to_string()) { continue; }

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