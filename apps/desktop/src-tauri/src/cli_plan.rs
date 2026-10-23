//! CLI customisation files and the text progress bar.

use imager_core::pipeline::{ProgressSnapshot, WriteState};

use super::cli::{Options, Plan};
use imager_core::customise::plan::CustomisationPlan;

/// `--cloudinit-userdata` / `--cloudinit-networkconfig` (init format
/// `cloudinit`) or `--first-run-script` (`systemd`), as upstream.
pub fn plan(options: &Options) -> Result<Plan, String> {
    let read = |key: &str, what: &str| -> Result<String, String> {
        let path = options.value_of(key);
        if path.is_empty() {
            return Ok(String::new());
        }
        if !std::path::Path::new(&path).exists() {
            return Err(format!("{what} file does not exists"));
        }
        std::fs::read_to_string(&path).map_err(|_| format!("opening {what} file"))
    };
    let (user_data, network) = (
        read("cloudinit-userdata", "user-data")?,
        read("cloudinit-networkconfig", "network-config")?,
    );
    if !user_data.is_empty() || !network.is_empty() {
        return Ok(Some(CustomisationPlan {
            init_format: "cloudinit".into(),
            cloudinit: user_data,
            network,
            ..CustomisationPlan::default()
        }));
    }
    let script = read("first-run-script", "firstrun script")?;
    let key = options.value_of("secure-boot-key");
    Ok(
        (!script.is_empty() || !key.is_empty()).then(|| CustomisationPlan {
            init_format: "systemd".into(),
            firstrun: script,
            secure_boot_key_path: key,
            ..CustomisationPlan::default()
        }),
    )
}

/// Upstream `_printProgress`: `  Writing: [----->     ] 25 %`.
#[derive(Default)]
pub struct Bar {
    last: Option<(&'static str, u64)>,
}

impl Bar {
    pub fn print(&mut self, s: &ProgressSnapshot) {
        let (label, now, total) = match s.state {
            WriteState::Verifying => ("Verifying", s.verify_now, s.verify_total),
            _ => (
                "Writing",
                s.bytes_written,
                s.write_total.max(s.download_total),
            ),
        };
        if total == 0 {
            return;
        }
        let percent = (now.saturating_mul(100) / total).min(100);
        if self.last != Some((label, percent)) {
            let done = (percent / 5) as usize;
            eprint!(
                "  {label}: [{}>{}] {percent} %\r",
                "-".repeat(done),
                " ".repeat(20 - done)
            );
            self.last = Some((label, percent));
        }
    }
}

impl Options {
    pub fn value_of(&self, name: &str) -> String {
        self.values
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    }
}
