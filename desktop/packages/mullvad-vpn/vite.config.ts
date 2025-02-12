import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import electron from 'vite-plugin-electron/simple';

export default defineConfig({
  define: {
    global: 'window',
    'process.env': {
      platform: process.platform,
      NODE_ENV: process.env.NODE_ENV,
    },
  },
  plugins: [
    electron({
      main: {
        entry: 'src/main/index.ts',
        vite: {
          build: {
            emptyOutDir: true,
            commonjsOptions: {
              include: [/management-interface/, /nseventforwarder/, /node_modules/],
            },
            rollupOptions: {
              output: {
                entryFileNames: 'main.js',
              },
              external: ['@grpc/grpc-js', 'google-protobuf'],
            },
          },
          optimizeDeps: {
            include: ['management-interface', 'nseventforwarder'],
          },
        },
      },
      preload: {
        input: 'src/renderer/preload.ts',
      },
    }),
    react(),
  ],
});
