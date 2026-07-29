import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("lucide-vue-next")) return "icons";
          if (
            id.includes("naive-ui") ||
            id.includes("vueuc") ||
            id.includes("css-render")
          ) {
            return "naive-ui";
          }
          if (id.includes("@tauri-apps")) return "tauri";
          if (id.includes("/vue/") || id.includes("@vue/")) return "vue";
        },
      },
    },
  },
});
