import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [
    react(),
    {
      name: "static-not-found-preview",
      configurePreviewServer(server) {
        server.middlewares.use((req, res, next) => {
          const pathname = new URL(req.url || "/", "http://localhost").pathname;
          const relative = pathname.replace(/^\//, "");
          if (
            relative &&
            !existsSync(resolve("dist", relative)) &&
            existsSync("dist/404.html")
          ) {
            res.statusCode = 404;
            res.setHeader("Content-Type", "text/html; charset=utf-8");
            res.end(readFileSync("dist/404.html"));
            return;
          }
          next();
        });
      },
    },
  ],
  base: "/",
});
