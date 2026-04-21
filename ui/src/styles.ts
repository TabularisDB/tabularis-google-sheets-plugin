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
    color: "var(--color-text-primary, #e2e8f0)",
  } as CSSProperties,
  section: {
    background: "var(--color-bg-elevated, #1e2533)",
    border: "1px solid var(--color-border, rgba(255,255,255,0.08))",
    borderRadius: "8px",
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
    color: "var(--color-text-primary, #e2e8f0)",
  } as CSSProperties,
  badge: (ok: boolean): CSSProperties => ({
    display: "inline-block",
    padding: "2px 8px",
    borderRadius: "12px",
    fontSize: "11px",
    fontWeight: 500,
    background: ok ? "rgba(34,197,94,0.15)" : "rgba(148,163,184,0.12)",
    color: ok ? "#4ade80" : "var(--color-text-muted, #94a3b8)",
  }),
  label: {
    display: "block",
    fontSize: "11px",
    fontWeight: 500,
    color: "var(--color-text-muted, #94a3b8)",
    marginBottom: "4px",
    marginTop: "10px",
  } as CSSProperties,
  input: {
    width: "100%",
    padding: "7px 10px",
    background: "var(--color-bg-base, #131929)",
    border: "1px solid var(--color-border, rgba(255,255,255,0.1))",
    borderRadius: "6px",
    color: "var(--color-text-primary, #e2e8f0)",
    fontSize: "12px",
    outline: "none",
    boxSizing: "border-box",
    fontFamily: "monospace",
  } as CSSProperties,
  hint: {
    fontSize: "11px",
    color: "var(--color-text-muted, #94a3b8)",
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
    borderRadius: "6px",
    fontSize: "12px",
    fontWeight: 500,
    cursor: "pointer",
    border: "none",
    outline: "none",
    transition: "opacity .15s",
    ...(variant === "primary"
      ? { background: "#3b82f6", color: "#fff" }
      : variant === "danger"
      ? {
          background: "rgba(239,68,68,0.15)",
          color: "#f87171",
          border: "1px solid rgba(239,68,68,0.25)",
        }
      : {
          background: "rgba(255,255,255,0.07)",
          color: "var(--color-text-primary, #e2e8f0)",
          border: "1px solid var(--color-border, rgba(255,255,255,0.1))",
        }),
  }),
  error: {
    marginTop: "8px",
    padding: "8px 10px",
    background: "rgba(239,68,68,0.1)",
    border: "1px solid rgba(239,68,68,0.2)",
    borderRadius: "6px",
    color: "#f87171",
    fontSize: "12px",
  } as CSSProperties,
  success: {
    marginTop: "8px",
    padding: "8px 10px",
    background: "rgba(34,197,94,0.1)",
    border: "1px solid rgba(34,197,94,0.2)",
    borderRadius: "6px",
    color: "#4ade80",
    fontSize: "12px",
  } as CSSProperties,
  steps: {
    marginTop: "10px",
    paddingLeft: "16px",
    lineHeight: 1.8,
    color: "var(--color-text-muted, #94a3b8)",
    fontSize: "12px",
  } as CSSProperties,
  codeBlock: {
    background: "var(--color-bg-base, #131929)",
    border: "1px solid var(--color-border, rgba(255,255,255,0.08))",
    borderRadius: "4px",
    padding: "4px 8px",
    fontFamily: "monospace",
    fontSize: "11px",
    display: "inline",
    color: "#93c5fd",
  } as CSSProperties,
};
