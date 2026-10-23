//! The `imager` command: QML-style `imageWriter.<method>(args…)` calls,
//! dispatched to small domain modules.

mod args;
mod drives;
mod lists;
mod locale;
mod misc;
mod settings;
mod source;
mod write;

use serde_json::Value;

pub use self::args::Args;
pub use self::lists::is_valid_repo_url;
pub use self::misc::is_connect_token;
use crate::state::State;

/// Runs one synchronous call against the state.
///
/// # Errors
/// Unknown methods and failing calls return a message.
pub fn dispatch(state: &mut State, method: &str, args: &[Value]) -> Result<Value, String> {
    let args = Args(args);
    settings::handle(state, method, &args)
        .or_else(|| lists::handle(state, method, &args))
        .or_else(|| source::handle(state, method, &args))
        .or_else(|| drives::handle(state, method, &args))
        .or_else(|| locale::handle(state, method, &args))
        .or_else(|| write::handle(state, method, &args))
        .or_else(|| misc::handle(state, method, &args))
        .unwrap_or_else(|| Err(format!("unknown imageWriter method: {method}")))
}
