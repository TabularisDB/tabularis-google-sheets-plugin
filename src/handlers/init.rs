//! `initialize` — the first RPC the host sends. Receives the plugin's
//! saved settings (client_id, client_secret, tokens) and pushes them into
//! the module-global auth state for later API calls.

use serde_json::Value;

use crate::auth::{auth, AuthState};
use crate::rpc::ok_response;

pub fn initialize(id: Value, params: &Value) -> Value {
    let settings = params.get("settings").cloned().unwrap_or(Value::Null);

    let mut state = auth().lock().unwrap();
    *state = AuthState::default();

    state.oauth_client_id = string_setting(&settings, "client_id");
    state.oauth_client_secret = string_setting(&settings, "client_secret");
    state.oauth_refresh_token = string_setting(&settings, "refresh_token");
    state.oauth_access_token = string_setting(&settings, "access_token");
    state.oauth_token_expiry = settings.get("token_expiry").and_then(Value::as_u64);

    ok_response(id, Value::Null)
}

fn string_setting(settings: &Value, key: &str) -> Option<String> {
    settings
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}
