import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Build the OAuth wizard as a single IIFE bundle. React,
// react/jsx-runtime and @tabularis/plugin-api are provided by the host
// as runtime globals.
export default defineConfig({
  plugins: [react()],
  build: {
    outDir: "dist",
    emptyOutDir: false,
    lib: {
      entry: "src/google-auth.tsx",
      formats: ["iife"],
      name: "__tabularis_plugin__",
      fileName: () => "google-auth.js",
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
