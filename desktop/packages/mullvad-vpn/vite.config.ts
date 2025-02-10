import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import { startup } from 'vite-plugin-electron';
import electron from 'vite-plugin-electron/simple';

import { treeKillSync } from './vite-utils';

// NOTE: We have to monkey patch the exit handler to override the default
// behavior for how to kill the electron app. We use a custom variant of the
// vite-plugin-electron's treeKillSync function to target only the electron
// application's process and its children and not the current behavior where
// the current process' children is targeted. This is because the current
// process spawns two processes, the electron app and esbuild.
//
// The default behavior of vite-plugin-electron when the electron app needs to
// restart is to kill both the electron app and the esbuild processes, however
// after that only the electron app gets respawned, leaving the esbuild process
// permanently dead after the first time the electron app has restarted.
//
// This should be fixed upstream but until then this is an okay workaround.
// As this is a hack I didn't bother fixing the types for process.electronApp
// correctly, hence the ts-ignore below.
startup.exit = async () => {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  const electronApp = process.electronApp;
  if (electronApp) {
    await new Promise((resolve) => {
      electronApp.removeAllListeners();
      electronApp.once('exit', resolve);
      treeKillSync(electronApp.pid);
    });
  }
};

const MAIN = process.env.NODE_ENV === 'test' ? 'test/e2e/setup/main.ts' : 'src/main/index.ts';
const OUT_DIR = 'build';

const viteConfig = defineConfig({
  define: {
    global: 'window',
    process: {
      platform: process.platform,
      env: {
        NODE_ENV: process.env.NODE_ENV,
      },
    },
  },
  build: {
    outDir: OUT_DIR,
  },
  plugins: [
    electron({
      main: {
        entry: MAIN,
        async onstart({ startup }) {
          // NOTE: vite-plugin-electron automatically adds --no-sandbox to its
          // command line arguments when spawning electron. From a security
          // standpoint this is not a good default so we omit it to allow
          // us setting it programmatically in the main process.
          //
          // Another consequence of the default --no-sandbox being added was
          // that it caused a crash when the devtools opened if the sandbox
          // had not been enabled again. However, after the default --no-sandbox
          // was omitted we can open the devtools regardless of whether the
          // sandbox is enabled or not.
          await startup(['.']);
        },
        vite: {
          build: {
            outDir: OUT_DIR,
            commonjsOptions: {
              include: [
                // Packages in workspace which exports common js
                /management-interface/,
                /nseventforwarder/,
                /win-shortcuts/,
                // External dependencies which exports common js
                /node_modules/,
              ],
            },
            rollupOptions: {
              output: {
                // We have to specify main.js here as otherwise it would
                // inherit the name from the entry file, i.e. index
                entryFileNames: 'main.js',
              },
              external: [
                // Packages in workspace which can not be bundled
                'win-shortcuts',
                // External dependencies
                '@grpc/grpc-js',
                'google-protobuf',
                'simple-plist',
              ],
            },
          },
          // Dependencies which can be transformed to e.g. become smaller or more efficient
          optimizeDeps: {
            include: ['management-interface', 'nseventforwarder'],
          },
        },
      },
      preload: {
        input: 'src/renderer/preload.ts',
        vite: {
          build: {
            outDir: OUT_DIR,
          },
        },
      },
    }),
    react(),
  ],
});

export default viteConfig;
