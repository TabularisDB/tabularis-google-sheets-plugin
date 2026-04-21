import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Build the connection-modal field as a separate IIFE bundle.
export default defineConfig({
  plugins: [react()],
  build: {
    outDir: "dist",
    emptyOutDir: false,
    lib: {
      entry: "src/google-sheets-db-field.tsx",
      formats: ["iife"],
      name: "__tabularis_plugin__",
      fileName: () => "google-sheets-db-field.js",
    },
    rollupOptions: {
      external: ["react", "react/jsx-runtime", "@tabularis/plugin-api"],
      output: {
        globals: {
          react: "React",
          "react/jsx-runtime": "ReactJSXRuntime",
          "@tabularis/plugin-api": "__TABULARIS_API__",
        },
      },
    },
  },
});
