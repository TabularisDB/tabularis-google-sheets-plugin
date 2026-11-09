/**
 * Shared inline styles for the Google Sheets UI extensions.
 * Kept as plain objects so both .tsx files can import them without
 * a CSS bundler step.
 */

import type { CSSProperties } from "react";

export const PLUGIN_ID = "google-sheets";

export const S = {
  wrap: {
    padding: "12px 0",
    fontSize: "13px",
    color: "var(--text-primary)",
  } as CSSProperties,
  section: {
    background: "var(--bg-elevated)",
    border: "1px solid var(--border-default)",
    borderRadius: "var(--radius-lg)",
    padding: "14px 16px",
    marginTop: "8px",
  } as CSSProperties,
  title: {
    fontWeight: 600,
    fontSize: "13px",
    marginBottom: "10px",
    display: "flex",
    alignItems: "center",
    gap: "7px",
    color: "var(--text-primary)",
  } as CSSProperties,
  badge: (ok: boolean): CSSProperties => ({
    display: "inline-block",
    padding: "2px 8px",
    borderRadius: "var(--radius-xl)",
    fontSize: "11px",
    fontWeight: 500,
    background: "var(--bg-elevated)",
    color: ok ? "var(--accent-success)" : "var(--text-muted)",
  }),
  label: {
    display: "block",
    fontSize: "11px",
    fontWeight: 500,
    color: "var(--text-muted)",
    marginBottom: "4px",
    marginTop: "10px",
  } as CSSProperties,
  input: {
    width: "100%",
    padding: "7px 10px",
    background: "var(--bg-base)",
    border: "1px solid var(--border-default)",
    borderRadius: "var(--radius-base)",
    color: "var(--text-primary)",
    fontSize: "12px",
    outlineColor: "var(--border-focus)",
    boxSizing: "border-box",
    fontFamily: "var(--font-mono)",
  } as CSSProperties,
  hint: {
    fontSize: "11px",
    color: "var(--text-muted)",
    marginTop: "4px",
    lineHeight: 1.5,
  } as CSSProperties,
  row: {
    display: "flex",
    gap: "8px",
    marginTop: "12px",
  } as CSSProperties,
  btn: (variant: "primary" | "danger" | "default"): CSSProperties => ({
    padding: "7px 14px",
    borderRadius: "var(--radius-base)",
    fontSize: "12px",
    fontWeight: 500,
    cursor: "pointer",
    border: "none",
    outlineColor: "var(--border-focus)",
    transition: "opacity .15s",
    ...(variant === "primary"
      ? { background: "var(--accent-primary)", color: "var(--text-inverse)" }
      : variant === "danger"
        ? {
            background: "var(--bg-elevated)",
            color: "var(--accent-error)",
            border: "1px solid var(--accent-error)",
          }
        : {
            background: "var(--surface-secondary)",
            color: "var(--text-primary)",
            border: "1px solid var(--border-default)",
          }),
  }),
  error: {
    marginTop: "8px",
    padding: "8px 10px",
    background: "var(--bg-elevated)",
    border: "1px solid var(--accent-error)",
    borderRadius: "var(--radius-base)",
    color: "var(--accent-error)",
    fontSize: "12px",
  } as CSSProperties,
  success: {
    marginTop: "8px",
    padding: "8px 10px",
    background: "var(--bg-elevated)",
    border: "1px solid var(--accent-success)",
    borderRadius: "var(--radius-base)",
    color: "var(--accent-success)",
    fontSize: "12px",
  } as CSSProperties,
  steps: {
    marginTop: "10px",
    paddingLeft: "16px",
    lineHeight: 1.8,
    color: "var(--text-muted)",
    fontSize: "12px",
  } as CSSProperties,
  codeBlock: {
    background: "var(--bg-base)",
    border: "1px solid var(--border-default)",
    borderRadius: "var(--radius-sm)",
    padding: "4px 8px",
    fontFamily: "var(--font-mono)",
    fontSize: "11px",
    display: "inline",
    color: "var(--text-accent)",
  } as CSSProperties,
};
