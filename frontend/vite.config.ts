import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { VitePWA } from "vite-plugin-pwa";

export default defineConfig(() => ({
	plugins: [
		react(),
		tailwindcss(),
		VitePWA({
			registerType: "prompt",
			includeAssets: ["favicon.svg", "robots.txt", "apple-touch-icon.png"],
			manifest: {
				name: "Cadence App",
				short_name: "Cadence",
				description: "Modern task management application",
				theme_color: "#ffffff",
				icons: [
					{
						src: "pwa-192x192.svg",
						sizes: "192x192",
						type: "image/svg+xml",
					},
					{
						src: "pwa-512x512.svg",
						sizes: "512x512",
						type: "image/svg+xml",
					},
					{
						src: "pwa-512x512.svg",
						sizes: "512x512",
						type: "image/svg+xml",
						purpose: "any maskable",
					},
				],
			},
			devOptions: {
				enabled: false,
			},
		}),
	],
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
}));
