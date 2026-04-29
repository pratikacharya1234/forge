import { defineConfig } from 'astro/config';
import svelte from '@astrojs/svelte';
import tailwindv4 from '@tailwindcss/vite';

export default defineConfig({
  integrations: [svelte()],
  output: 'static',
  vite: {
    plugins: [tailwindv4()],
    server: {
      headers: {
        'Content-Security-Policy': "default-src 'self'; script-src 'self' 'unsafe-inline' https://api.counterapi.dev; style-src 'self' 'unsafe-inline'; img-src 'self' data: https://api.github.com; connect-src 'self' https://api.github.com https://api.counterapi.dev;",
      }
    }
  },
});
