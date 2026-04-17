import { resolve } from "node:path";

import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  base: "/assets/dist/",
  plugins: [react()],
  test: {
    environment: "jsdom",
    environmentOptions: {
      jsdom: {
        url: "http://localhost/",
      },
    },
    include: ["src/**/*.test.ts"],
  },
  resolve: {
    dedupe: ["react", "react-dom"],
    alias: [
      { find: "react", replacement: resolve(__dirname, "node_modules/react") },
      {
        find: "react-dom",
        replacement: resolve(__dirname, "node_modules/react-dom"),
      },
    ],
  },
  build: {
    manifest: "manifest.json",
    outDir: resolve(__dirname, "../assets/dist"),
    emptyOutDir: true,
    rollupOptions: {
      input: {
        "auth/signin.html": resolve(__dirname, "auth/signin.html"),
        "auth/signup.html": resolve(__dirname, "auth/signup.html"),
        "app/index-admin.html": resolve(__dirname, "app/index-admin.html"),
        "app/index-basic.html": resolve(__dirname, "app/index-basic.html"),
        "src/entries/auth-signin.tsx": resolve(
          __dirname,
          "src/entries/auth-signin.tsx",
        ),
        "src/entries/auth-signup.tsx": resolve(
          __dirname,
          "src/entries/auth-signup.tsx",
        ),
        "src/entries/main-admin.tsx": resolve(
          __dirname,
          "src/entries/main-admin.tsx",
        ),
        "src/entries/main-basic.tsx": resolve(
          __dirname,
          "src/entries/main-basic.tsx",
        ),
      },
    },
  },
});
