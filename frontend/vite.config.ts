import { resolve } from "node:path";

import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],
  build: {
    manifest: "manifest.json",
    outDir: resolve(__dirname, "../assets/dist"),
    emptyOutDir: true,
    rollupOptions: {
      input: [
        resolve(__dirname, "src/entries/auth-signin.tsx"),
        resolve(__dirname, "src/entries/auth-signup.tsx"),
        resolve(__dirname, "src/entries/main-admin.tsx"),
        resolve(__dirname, "src/entries/main-basic.tsx"),
      ],
    },
  },
});
