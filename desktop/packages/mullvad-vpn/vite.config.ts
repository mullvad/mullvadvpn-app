import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import electron from 'vite-plugin-electron/simple';

export default defineConfig({
  define: {
    global: 'window',
  },
  plugins: [
    electron({
      main: {
        entry: 'src/main/index.ts',
        vite: {
          optimizeDeps: {
            include: ['management-interface', 'nseventforwarder'],
          },
          build: {
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
        },
      },
      preload: {
        input: 'src/renderer/preload.ts',
      },
    }),
    react(),
  ],
});
