import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Served from GitHub Pages at /bitgate/.
export default defineConfig({
  base: "./",
  plugins: [react()],
  build: { target: "es2022", chunkSizeWarningLimit: 2000 },
});
