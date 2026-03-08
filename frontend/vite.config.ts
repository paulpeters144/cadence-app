import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
	plugins: [react(), tailwindcss()],
	resolve: {
		alias: {
			"@": path.resolve(import.meta.dirname, "./src"),
		},
	},
	define: {
		"import.meta.env.VITE_APP_BUILD_TIME": JSON.stringify(
			new Date().toISOString().slice(0, 16),
		),
	},
});
