//! Settings calls (`getBoolSetting`, `setSetting`, saved customisation).

use imager_core::settings::CUSTOMISATION_GROUP;
use serde_json::Value;

use super::args::{Args, Handled, done, ok};
use crate::state::State;

pub fn handle(state: &mut State, method: &str, args: &Args<'_>) -> Handled {
    let settings = &mut state.settings;
    match method {
        "getBoolSetting" => ok(settings.bool(&args.str(0))),
        "getStringSetting" => ok(settings.string(&args.str(0))),
        "setSetting" => done(
            settings
                .set(&args.str(0), args.value(1))
                .map(|()| Value::Null),
        ),
        "getSavedCustomisationSettings" => ok(Value::Object(settings.customisation())),
        "setSavedCustomisationSettings" => {
            let map = args.value(0).as_object().cloned().unwrap_or_default();
            done(settings.set_customisation(map).map(|()| Value::Null))
        }
        "setPersistedCustomisationSetting" => done(
            settings
                .set_group_value(CUSTOMISATION_GROUP, &args.str(0), args.value(1))
                .map(|()| Value::Null),
        ),
        "removePersistedCustomisationSetting" => done(
            settings
                .remove_group_value(CUSTOMISATION_GROUP, &args.str(0))
                .map(|()| Value::Null),
        ),
        "clearSavedCustomisationSettings" => {
            done(settings.clear_customisation().map(|()| Value::Null))
        }
        _ => None,
    }
}
