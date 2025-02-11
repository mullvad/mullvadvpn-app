import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import electron from 'vite-plugin-electron/simple';

const outDir = process.env.NODE_ENV === 'development' ? 'dist-dev' : 'dist-prod';

export default defineConfig({
  define: {
    global: 'window',
  },
  build: {
    outDir,
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
            outDir,
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
        vite: {
          build: {
            outDir,
          },
        },
        input: 'src/renderer/preload.ts',
      },
    }),
    react(),
  ],
});
