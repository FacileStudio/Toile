import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

const api = 'http://127.0.0.1:4000';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	optimizeDeps: {
		exclude: ['@facile/muse', '@facile/client']
	},
	resolve: process.env.VITEST ? { conditions: ['browser'] } : {},
	server: {
		proxy: { '/api': { target: api, changeOrigin: true, ws: true } }
	},
	test: {
		include: ['src/**/*.test.ts'],
		environment: 'jsdom'
	}
});
