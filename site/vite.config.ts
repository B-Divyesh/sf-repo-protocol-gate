import { defineConfig } from "vite";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  root: fileURLToPath(new URL(".", import.meta.url)),
  publicDir: "public",
  build: {
    outDir: "../dist/site",
    emptyOutDir: true,
    target: "es2022",
    sourcemap: false,
  },
});
