import { defineConfig } from 'vite';

export default defineConfig({
  root: 'src',
  // Relative base so the packaged build's assets resolve under file://
  base: './',
  build: {
    outDir: '../dist',
    emptyOutDir: true
  },
  server: {
    port: 5189,
    strictPort: true
  }
});
