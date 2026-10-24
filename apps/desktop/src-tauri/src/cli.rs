//! `--cli` mode (upstream `cli.cpp`): `etchwise --cli [options] <src> <dst>`
//! writes without the GUI, printing upstream's progress bar to stderr.

use std::sync::Arc;
use std::time::Duration;

use imager_core::customise::plan::CustomisationPlan;
use imager_core::drives::visible_drives;
use imager_core::pipeline::job::{Job, Source};
use imager_core::pipeline::{Progress, WriteOptions, WriteState};

const HELP: &str = "Usage: etchwise --cli [options] src dst

Options:
  --disable-verify                    Disable verification
  --enable-writing-system-drives      Only use this if you know what you are doing
  --sha256 <sha256>                   Expected hash
  --cache-file <cache-file>           Custom cache file (requires setting sha256 as well)
  --first-run-script <script>         Add firstrun.sh to image
  --cloudinit-userdata <file>         Add cloud-init user-data file to image
  --cloudinit-networkconfig <file>    Add cloud-init network-config file to image
  --disable-eject                     Disable automatic ejection of storage media after verification
  --quiet                             Only write to console on error
  --secure-boot-key <key-file>        Path to RSA private key (PEM format) for secure boot signing

Arguments:
  src                                 Image file/URL
  dst                                 Destination device";

/// Parsed command line.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub positional: Vec<String>,
    pub flags: Vec<String>,
    pub values: Vec<(String, String)>,
}

impl Options {
    fn flag(&self, name: &str) -> bool {
        self.flags.iter().any(|f| f == name)
    }
}

const VALUED: [&str; 7] = [
    "sha256",
    "cache-file",
    "first-run-script",
    "cloudinit-userdata",
    "cloudinit-networkconfig",
    "log-file",
    "secure-boot-key",
];

/// Splits `args` (after the program name) into flags, values and positionals.
pub fn parse(args: &[String]) -> Options {
    let mut options = Options::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.strip_prefix("--") {
            Some(name) if VALUED.contains(&name) => options
                .values
                .push((name.to_owned(), iter.next().cloned().unwrap_or_default())),
            Some(name) => options.flags.push(name.to_owned()),
            None => options.positional.push(arg.clone()),
        }
    }
    options
}

/// Runs the CLI; returns the process exit code.
pub fn run(args: &[String]) -> i32 {
    let options = parse(args);
    if options.flag("help") || options.positional.len() != 2 {
        eprintln!("{HELP}");
        return i32::from(!options.flag("help"));
    }
    if options.flag("version") {
        eprintln!("{}", imager_core::IMAGER_VERSION);
        return 0;
    }
    match build(&options).and_then(|(job, dst)| {
        write(
            &job,
            &dst,
            options.flag("quiet"),
            !options.flag("disable-eject"),
        )
    }) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("\x1b[0KError: {message}");
            1
        }
    }
}

fn build(options: &Options) -> Result<(Job, String), String> {
    let (src, dst) = (&options.positional[0], options.positional[1].clone());
    let source = if src.to_lowercase().starts_with("http:")
        || src.to_lowercase().starts_with("https:")
    {
        Source::Remote(src.clone())
    } else {
        let meta = std::fs::metadata(src).map_err(|_| "source file does not exists".to_owned())?;
        if !meta.is_file() {
            return Err("source is not a regular file".into());
        }
        Source::Local(src.into())
    };
    check_destination(options, &dst)?;
    let sha = options.value_of("sha256");
    let plan = super::cli_plan::plan(options)?;
    let job = Job {
        source: Some(source),
        write: WriteOptions {
            expected_sha256: (!sha.is_empty()).then_some(sha),
            ..WriteOptions::default()
        },
        verify: !options.flag("disable-verify"),
        plan,
        ..Job::default()
    };
    Ok((job, dst))
}

fn check_destination(options: &Options, dst: &str) -> Result<(), String> {
    if options.flag("enable-writing-system-drives") {
        eprintln!("WARNING: writing to system drives is enabled.");
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    let drives = visible_drives(
        &imager_platform::macos::drivelist::list_drives().map_err(|e| e.to_string())?,
    );
    #[cfg(not(target_os = "macos"))]
    let drives = visible_drives(&[]);
    if drives.iter().any(|d| d.device == dst) {
        return Ok(());
    }
    let listing: String = drives
        .iter()
        .map(|d| format!("{} ({})\n", d.device, d.description))
        .collect();
    Err(format!(
        "Destination drive is not in list of removable volumes. Choose one of the following:\n\n{listing}\nOr use --enable-writing-system-drives to overrule."
    ))
}

fn write(job: &Job, dst: &str, quiet: bool, eject: bool) -> Result<(), String> {
    let progress = Arc::new(Progress::default());
    let worker = {
        let (job, dst, progress) = (job.clone(), dst.to_owned(), Arc::clone(&progress));
        std::thread::spawn(move || super::write_job::device_ops::run(&dst, &job, &progress))
    };
    let mut bar = super::cli_plan::Bar::default();
    while !worker.is_finished() {
        if !quiet {
            bar.print(&progress.snapshot());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    worker
        .join()
        .map_err(|_| "write thread panicked".to_owned())??;
    if eject {
        super::write_job::device_ops::after_success(dst, true, false);
    }
    if !quiet && progress.state() == WriteState::Succeeded {
        eprintln!("\x1b[0KWrite successful.");
    }
    Ok(())
}

/// Default plan type for callers outside the GUI.
pub type Plan = Option<CustomisationPlan>;

#[cfg(test)]
mod tests;
