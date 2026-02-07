import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://dmk.github.io',
  base: '/artbox',
  integrations: [
    starlight({
      title: 'artbox',
      description: 'Render FIGlet/ASCII text into a bounded rectangle with colors and gradients',
      social: {
        github: 'https://github.com/dmk/artbox',
      },
      editLink: {
        baseUrl: 'https://github.com/dmk/artbox/edit/main/docs/',
      },
      sidebar: [
        {
          label: 'Getting Started',
          items: [{ label: 'Quick Start', slug: 'getting-started/quick-start' }],
        },
        {
          label: 'Guides',
          items: [
            { label: 'Text and Fonts', slug: 'guides/text-and-fonts' },
            { label: 'Colors and Gradients', slug: 'guides/colors-and-gradients' },
            { label: 'Sprites', slug: 'guides/sprites' },
            { label: 'Images', slug: 'guides/images' },
            { label: 'Ratatui', slug: 'guides/ratatui' },
            { label: 'CLI', slug: 'guides/cli' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'API Overview', slug: 'reference/api' },
            { label: 'Feature Flags', slug: 'reference/feature-flags' },
          ],
        },
      ],
    }),
  ],
});
