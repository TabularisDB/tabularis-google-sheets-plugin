/**
 * UI extension — connection-modal.connection_content
 *
 * Replaces the default host/port/user/pass form in the "new connection"
 * modal with a single "Spreadsheet ID or URL" input. Active only when the
 * driver being configured is google-sheets.
 */

import { defineSlot } from "@tabularis/plugin-api";
import type { TypedSlotProps } from "@tabularis/plugin-api";
import type { ChangeEvent } from "react";

import { PLUGIN_ID } from "./styles";

// The slot's runtime context includes `database` and `onDatabaseChange`
// alongside the typed `driver`. Declare them locally until plugin-api
// tightens the shape.
type FieldContext = TypedSlotProps<"connection-modal.connection_content">["context"] & {
  database?: string;
  onDatabaseChange?: (value: string) => void;
};

const GoogleSheetsDatabaseField = defineSlot(
  "connection-modal.connection_content",
  ({ context }) => {
    const c = context as FieldContext;

    if (c.driver !== PLUGIN_ID) return null;

    const value = typeof c.database === "string" ? c.database : "";
    const onChange = c.onDatabaseChange ?? (() => {});

    return (
      <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
        <label
          style={{
            fontSize: "10px",
            textTransform: "uppercase",
            fontWeight: 600,
            letterSpacing: "0.05em",
            color: "var(--color-text-muted, #94a3b8)",
          }}
        >
          Spreadsheet ID or URL
        </label>
        <input
          type="text"
          value={value}
          onChange={(e: ChangeEvent<HTMLInputElement>) => onChange(e.target.value)}
          autoCorrect="off"
          autoCapitalize="off"
          autoComplete="off"
          spellCheck={false}
          placeholder="https://docs.google.com/spreadsheets/d/… or spreadsheet ID"
          style={{
            width: "100%",
            padding: "7px 10px",
            background: "var(--color-bg-base, #131929)",
            border: "1px solid rgba(255,255,255,0.15)",
            borderRadius: "6px",
            color: "var(--color-text-primary, #e2e8f0)",
            fontSize: "13px",
            outline: "none",
            boxSizing: "border-box",
          }}
        />
        <p
          style={{
            fontSize: "11px",
            color: "var(--color-text-muted, #94a3b8)",
            marginTop: "2px",
          }}
        >
          Each tab in the spreadsheet becomes a queryable table.
        </p>
      </div>
    );
  },
);

export default GoogleSheetsDatabaseField.component;
