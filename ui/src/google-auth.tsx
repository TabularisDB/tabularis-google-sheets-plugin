/**
 * UI extension — settings.plugin.before_settings
 *
 * Renders an OAuth2 setup wizard above the Google Sheets plugin's
 * settings form. Uses the typed hooks from @tabularis/plugin-api:
 *   - usePluginSetting(pluginId) — persist tokens across restarts
 *   - usePluginModal()           — two-step wizard modal
 *   - openUrl(url)               — launch the Google consent page in the
 *                                  system browser (Tauri webview doesn't
 *                                  open external URLs)
 */

import {
  defineSlot,
  openUrl,
  usePluginModal,
  usePluginSetting,
} from "@tabularis/plugin-api";
import { useState } from "react";

import { PLUGIN_ID, S } from "./styles";

const SCOPE = "https://www.googleapis.com/auth/spreadsheets";
const TOKEN_URL = "https://oauth2.googleapis.com/token";
const REDIRECT_URI = "http://127.0.0.1";

function buildAuthUrl(clientId: string): string {
  const params = new URLSearchParams({
    client_id: clientId,
    redirect_uri: REDIRECT_URI,
    response_type: "code",
    scope: SCOPE,
    access_type: "offline",
    prompt: "consent",
  });
  return "https://accounts.google.com/o/oauth2/v2/auth?" + params.toString();
}

/** Accept either a bare authorization code or the full redirect URL. */
function extractCode(input: string): string {
  const trimmed = input.trim();
  try {
    const url = new URL(trimmed);
    const code = url.searchParams.get("code");
    if (code) return code;
  } catch {
    // Not a URL — fall through and treat the whole string as the code.
  }
  return trimmed;
}

interface TokenResponse {
  access_token: string;
  refresh_token?: string;
  expires_in?: number;
}

async function exchangeCode(
  clientId: string,
  clientSecret: string,
  code: string,
): Promise<TokenResponse> {
  const body = new URLSearchParams({
    code,
    client_id: clientId,
    client_secret: clientSecret,
    redirect_uri: REDIRECT_URI,
    grant_type: "authorization_code",
  });
  const resp = await fetch(TOKEN_URL, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: body.toString(),
  });
  if (!resp.ok) {
    const text = await resp.text();
    let msg = "Token exchange failed";
    try {
      const json = JSON.parse(text) as {
        error_description?: string;
        error?: string;
      };
      msg = json.error_description || json.error || msg;
    } catch {
      msg = text || msg;
    }
    throw new Error(msg);
  }
  return resp.json() as Promise<TokenResponse>;
}

// ---------------------------------------------------------------------------
// Modal — two-step wizard
// ---------------------------------------------------------------------------

interface WizardProps {
  getSetting: <T = unknown>(key: string, defaultValue?: T) => T;
  setSetting: (key: string, value: unknown) => void;
  setSettings: (updates: Record<string, unknown>) => void;
  onClose: () => void;
}

function OAuthWizard({ getSetting, setSetting, setSettings, onClose }: WizardProps) {
  const [clientId, setClientId] = useState<string>(
    (getSetting<string>("client_id", "") as string) || "",
  );
  const [clientSecret, setClientSecret] = useState<string>(
    (getSetting<string>("client_secret", "") as string) || "",
  );
  const [authCode, setAuthCode] = useState("");
  const [step, setStep] = useState<"credentials" | "waiting" | "done">(
    "credentials",
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const canOpenUrl =
    clientId.trim().length > 0 && clientSecret.trim().length > 0;

  function handleOpenBrowser() {
    const url = buildAuthUrl(clientId.trim());
    setSetting("client_id", clientId.trim());
    setSetting("client_secret", clientSecret.trim());
    openUrl(url);
    setStep("waiting");
    setError("");
  }

  async function handleExchange() {
    const code = extractCode(authCode);
    if (!code) {
      setError("Please paste the redirect URL or authorization code.");
      return;
    }
    setLoading(true);
    setError("");
    try {
      const data = await exchangeCode(clientId.trim(), clientSecret.trim(), code);
      const expiry = Math.floor(Date.now() / 1000) + (data.expires_in ?? 3600);
      const updates: Record<string, unknown> = {
        client_id: clientId.trim(),
        client_secret: clientSecret.trim(),
        access_token: data.access_token,
        token_expiry: expiry,
      };
      if (data.refresh_token) {
        updates.refresh_token = data.refresh_token;
      }
      setSettings(updates);
      setStep("done");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  }

  if (step === "done") {
    return (
      <div>
        <div style={S.success}>
          ✓ Connected successfully! You can now create a Google Sheets connection.
        </div>
        <div style={S.row}>
          <button style={S.btn("default")} onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    );
  }

  const heading =
    step === "credentials"
      ? "Step 1 — Google OAuth credentials"
      : "Step 2 — Authorize and paste the code";

  return (
    <div style={{ minWidth: "460px" }}>
      <div style={{ marginBottom: "4px" }}>
        <div
          style={{
            fontWeight: 600,
            fontSize: "13px",
            marginBottom: "12px",
            color: "var(--color-text-primary, #e2e8f0)",
          }}
        >
          {heading}
        </div>
      </div>

      {step === "credentials" && (
        <div>
          <div style={S.hint}>
            Create OAuth 2.0 credentials in Google Cloud Console:
            <br />
            APIs &amp; Services → Credentials → Create → OAuth client ID →{" "}
            <span style={S.codeBlock}>Desktop app</span>
          </div>

          <label style={S.label}>Client ID</label>
          <input
            style={S.input}
            type="text"
            placeholder="123456789-abc....apps.googleusercontent.com"
            value={clientId}
            onChange={(e) => setClientId(e.target.value)}
            autoComplete="off"
          />

          <label style={S.label}>Client Secret</label>
          <input
            style={S.input}
            type="password"
            placeholder="GOCSPX-..."
            value={clientSecret}
            onChange={(e) => setClientSecret(e.target.value)}
            autoComplete="off"
          />

          {error && <div style={S.error}>{error}</div>}

          <div style={S.row}>
            <button
              style={{
                ...S.btn("primary"),
                opacity: canOpenUrl ? 1 : 0.4,
                cursor: canOpenUrl ? "pointer" : "not-allowed",
              }}
              disabled={!canOpenUrl}
              onClick={handleOpenBrowser}
            >
              Open Authorization Page →
            </button>
            <button style={S.btn("default")} onClick={onClose}>
              Cancel
            </button>
          </div>
        </div>
      )}

      {step === "waiting" && (
        <div>
          <ol style={S.steps}>
            <li>A Google sign-in page should have opened in your browser.</li>
            <li>Grant access to Google Sheets.</li>
            <li>
              After clicking "Allow", your browser will try to open{" "}
              <span style={S.codeBlock}>http://127.0.0.1/</span> (which will
              fail — that's expected).
            </li>
            <li>
              Copy the full URL from the address bar and paste it below{" "}
              <em>(it contains the authorization code).</em>
            </li>
          </ol>

          <label style={S.label}>Paste redirect URL or authorization code</label>
          <input
            style={{ ...S.input, fontFamily: "monospace" }}
            type="text"
            placeholder="http://127.0.0.1/?code=4/0AX4XfW...  or just the code"
            value={authCode}
            onChange={(e) => setAuthCode(e.target.value)}
            autoComplete="off"
            spellCheck={false}
          />

          {error && <div style={S.error}>{error}</div>}

          <div style={S.row}>
            <button
              style={{
                ...S.btn("primary"),
                opacity: loading ? 0.6 : 1,
                cursor: loading ? "not-allowed" : "pointer",
              }}
              disabled={loading}
              onClick={() => {
                void handleExchange();
              }}
            >
              {loading ? "Connecting…" : "Save Token"}
            </button>
            <button
              style={S.btn("default")}
              onClick={() => {
                setStep("credentials");
                setError("");
              }}
            >
              ← Back
            </button>
          </div>

          <div style={{ marginTop: "12px" }}>
            <button
              style={{ ...S.btn("default"), fontSize: "11px" }}
              onClick={handleOpenBrowser}
            >
              Re-open authorization page
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Slot component
// ---------------------------------------------------------------------------

const GoogleSheetsOAuth = defineSlot(
  "settings.plugin.before_settings",
  ({ context }) => {
    if (context.targetPluginId !== PLUGIN_ID) return null;

    const { getSetting, setSetting, setSettings } = usePluginSetting(PLUGIN_ID);
    const { openModal, closeModal } = usePluginModal();

    const refresh = getSetting<string>("refresh_token", "") as string;
    const access = getSetting<string>("access_token", "") as string;
    const clientId = (getSetting<string>("client_id", "") as string) || "";
    const isConnected = Boolean(refresh || access);

    function handleOpenOAuth() {
      openModal({
        title: "Connect Google Account",
        size: "md",
        content: (
          <OAuthWizard
            getSetting={getSetting}
            setSetting={setSetting}
            setSettings={setSettings}
            onClose={closeModal}
          />
        ),
      });
    }

    function handleDisconnect() {
      setSettings({
        access_token: "",
        refresh_token: "",
        token_expiry: 0,
      });
    }

    return (
      <div style={S.wrap}>
        <div style={S.section}>
          <div style={S.title}>
            <span>Google Account</span>
            <span style={S.badge(isConnected)}>
              {isConnected ? "Connected" : "Not connected"}
            </span>
          </div>

          {isConnected ? (
            <div>
              <div style={S.hint}>
                OAuth tokens are active. The plugin will refresh them automatically.
                {clientId && (
                  <span>
                    {" "}Client ID:{" "}
                    <span style={S.codeBlock}>{clientId.slice(0, 20) + "…"}</span>
                  </span>
                )}
              </div>
              <div style={S.row}>
                <button style={S.btn("default")} onClick={handleOpenOAuth}>
                  Re-authorize
                </button>
                <button style={S.btn("danger")} onClick={handleDisconnect}>
                  Disconnect
                </button>
              </div>
            </div>
          ) : (
            <div>
              <div style={S.hint}>
                Connect a Google account to access private sheets and enable
                write operations.
              </div>
              <div style={S.row}>
                <button style={S.btn("primary")} onClick={handleOpenOAuth}>
                  Connect with Google →
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    );
  },
);

export default GoogleSheetsOAuth.component;
