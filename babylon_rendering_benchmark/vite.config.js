import { defineConfig } from "vite";

// Name of the computer for the CSV, set when building: `PC_NAME="desktop" npm run build`
export default defineConfig({
  base: "./",
  define: {
    __PC_NAME__: JSON.stringify(process.env.PC_NAME ?? "unknown"),
  },
  server: { port: 8083 },
  preview: { port: 8083 },
});
