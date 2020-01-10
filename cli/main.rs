use nostd_check::{check, DependencyCheckResult};

fn main() -> Result<(), &'static str> {
    let mut ok = true;
    for summary in check() {
        match summary.result {
            DependencyCheckResult::Ok => {}, //println!("{}: ok", summary.package_id),
            DependencyCheckResult::IsPurelyStd => {
                println!("{}: ERROR - PURELY STD", summary.package_id);
                ok = false;
            },
            DependencyCheckResult::IsConditionallyNoStdButStdIsActivated => {
                println!("{}: ERROR - DEACTIVATE STD", summary.package_id);
                ok = false;
            },
            DependencyCheckResult::Unknown => {
                println!("{}: UNKNOWN (cannot parse)", summary.package_id);
            }
        }
    }

    if !ok { Err("Some of the dependecies failed the check, see above output") } else { Ok(())}
}