import { defineConfig } from 'astro/config';
import tailwind from '@astrojs/tailwind';
import mdx from '@astrojs/mdx';
import sitemap from '@astrojs/sitemap';

export default defineConfig({
  site: 'https://copyleftdev.github.io',
  base: '/sigmos',
  integrations: [
    tailwind(),
    mdx(),
    sitemap()
  ],
  markdown: {
    shikiConfig: {
      theme: 'github-dark-dimmed',
      langs: ['javascript', 'typescript', 'rust', 'bash', 'json', 'yaml'],
      wrap: true
    }
  },
  vite: {
    ssr: {
      noExternal: ['@astrojs/prism']
    }
  }
});
