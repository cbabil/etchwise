use super::*;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn parses_flags_values_and_positionals() {
    let o = parse(&args(&[
        "--disable-verify",
        "--sha256",
        "abc",
        "img.xz",
        "/dev/disk4",
        "--quiet",
    ]));
    assert_eq!(o.positional, ["img.xz", "/dev/disk4"]);
    assert!(o.flag("disable-verify") && o.flag("quiet") && !o.flag("debug"));
    assert_eq!(o.value_of("sha256"), "abc");
    assert_eq!(o.value_of("cache-file"), "");
}

#[test]
fn rejects_missing_or_irregular_sources() {
    assert_eq!(run(&args(&["only-one"])), 1);
    assert_eq!(run(&args(&["--help"])), 0);
    let err = build(&parse(&args(&["/nonexistent/x.img", "/dev/disk9"]))).unwrap_err();
    assert_eq!(err, "source file does not exists");
    let dir = tempfile::tempdir().unwrap();
    let err = build(&parse(&args(&[dir.path().to_str().unwrap(), "/dev/disk9"]))).unwrap_err();
    assert_eq!(err, "source is not a regular file");
}

#[test]
fn refuses_unknown_destinations_unless_overridden() {
    let err = check_destination(&parse(&args(&["a", "/dev/disk999"])), "/dev/disk999").unwrap_err();
    assert!(
        err.starts_with("Destination drive is not in list of removable volumes."),
        "{err}"
    );
    check_destination(
        &parse(&args(&["--enable-writing-system-drives"])),
        "/dev/disk999",
    )
    .unwrap();
}

#[test]
fn builds_customisation_plans() {
    let dir = tempfile::tempdir().unwrap();
    let ud = dir.path().join("user-data");
    std::fs::write(&ud, "hostname: pi\n").unwrap();
    let o = parse(&args(&["--cloudinit-userdata", ud.to_str().unwrap()]));
    let plan = super::super::cli_plan::plan(&o).unwrap().unwrap();
    assert_eq!(
        (plan.init_format.as_str(), plan.cloudinit.as_str()),
        ("cloudinit", "hostname: pi\n")
    );
    assert!(super::super::cli_plan::plan(&parse(&[])).unwrap().is_none());
    let missing = parse(&args(&["--first-run-script", "/nonexistent.sh"]));
    assert_eq!(
        super::super::cli_plan::plan(&missing).unwrap_err(),
        "firstrun script file does not exists"
    );
}
