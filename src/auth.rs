use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Global auth state
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct AuthState {
    // OAuth2
    pub oauth_client_id: Option<String>,
    pub oauth_client_secret: Option<String>,
    pub oauth_access_token: Option<String>,
    pub oauth_refresh_token: Option<String>,
    pub oauth_token_expiry: Option<u64>, // unix seconds
}

static AUTH: OnceLock<Mutex<AuthState>> = OnceLock::new();

pub fn auth() -> &'static Mutex<AuthState> {
    AUTH.get_or_init(|| Mutex::new(AuthState::default()))
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/// Returns the current OAuth2 access token, refreshing it if needed.
pub fn access_token(client: &reqwest::blocking::Client) -> anyhow::Result<String> {
    let state = auth().lock().unwrap();

    if state.oauth_refresh_token.is_none() && state.oauth_access_token.is_none() {
        anyhow::bail!(
            "No credentials configured. \
             Open Settings → Plugins → Google Sheets and connect your account."
        );
    }

    let now = unix_now();

    // Use cached access token if still valid
    if let Some(ref token) = state.oauth_access_token.clone() {
        let expiry = state.oauth_token_expiry.unwrap_or(0);
        if now + 60 < expiry {
            return Ok(token.clone());
        }
    }

    // Refresh using refresh_token
    let refresh_token = state
        .oauth_refresh_token
        .clone()
        .ok_or_else(|| anyhow::anyhow!(
            "OAuth access token expired and no refresh token available. \
             Re-authorize in Settings → Plugins → Google Sheets."
        ))?;
    let client_id = state
        .oauth_client_id
        .clone()
        .ok_or_else(|| anyhow::anyhow!("OAuth client_id not set"))?;
    let client_secret = state
        .oauth_client_secret
        .clone()
        .ok_or_else(|| anyhow::anyhow!("OAuth client_secret not set"))?;

    drop(state);

    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh_token),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
        ])
        .send()?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("OAuth token refresh failed ({status}): {body}");
    }

    let resp: TokenResponse = response.json()?;

    let new_token = resp.access_token.clone();
    let expiry = unix_now() + resp.expires_in;

    let mut state = auth().lock().unwrap();
    state.oauth_access_token = Some(new_token.clone());
    state.oauth_token_expiry = Some(expiry);

    Ok(new_token)
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
