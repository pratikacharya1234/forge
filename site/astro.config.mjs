import { defineConfig } from 'astro/config';
import svelte from '@astrojs/svelte';
import tailwindv4 from '@tailwindcss/vite';

export default defineConfig({
  integrations: [svelte()],
  vite: {
    plugins: [tailwindv4()],
  },
});
