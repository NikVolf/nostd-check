
#[test]
fn local_explore() {

    let deps = crate::find_all_root_libs();

    let rustc_workspace_hack = deps.iter()
        .find(|x| x.package_id.to_string() == "rustc-workspace-hack v1.0.0")
        .expect("There should be this dependency")
        .clone();

    println!("rustc_workspace_hack libpath: {}", rustc_workspace_hack.lib_path.to_string_lossy());
    assert_eq!(crate::explore(rustc_workspace_hack.lib_path), crate::Exploration::Std);

    let bitflags = deps.iter()
        .find(|x| x.package_id.to_string() == "bitflags v1.2.1")
        .expect("There should be this dependency")
        .clone();

    println!("bitflags libpath: {}", bitflags.lib_path.to_string_lossy());
    assert_eq!(crate::explore(bitflags.lib_path), crate::Exploration::NoStd);

}