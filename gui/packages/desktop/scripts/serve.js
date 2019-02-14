const { spawn } = require('child_process');
const path = require('path');
const TscWatchClient = require('tsc-watch/client');
const electron = require('electron');
const browserSync = require('browser-sync');
const browserSyncConnectUtils = require('browser-sync/dist/connect-utils');
const bsync = browserSync.create();

const getRootUrl = (options) => {
  const port = options.get('port');
  return `http://localhost:${port}`;
};

const getClientUrl = (options) => {
  const pathname = browserSyncConnectUtils.clientScript(options);
  return getRootUrl(options) + pathname;
};

function runElectron(browserSyncUrl) {
  const child = spawn(electron, ['.', '--enable-logging'], {
    env: {
      ...{
        NODE_ENV: 'development',
        BROWSER_SYNC_CLIENT_URL: browserSyncUrl,
      },
      ...process.env,
    },
    stdio: 'inherit',
  });
  child.once('close', () => {
    process.exit();
  });

  return child;
}

function startBrowserSync() {
  bsync.init(
    {
      ui: false,
      // Port 35829 = LiveReload's default port 35729 + 100.
      // If the port is occupied, Browsersync uses next free port automatically.
      port: 35829,
      ghostMode: false,
      open: false,
      notify: false,
      logSnippet: false,
      socket: {
        // Use the actual port here.
        domain: getRootUrl,
      },
    },
    (err, bs) => {
      if (err) return console.error(err);

      const browserSyncUrl = getClientUrl(bs.options);

      let child = runElectron(browserSyncUrl);

      bsync
        .watch(['build/src/config.json', 'build/src/main/**/*', 'build/src/shared/**/*'])
        .on('change', () => {
          child.removeAllListeners('close');
          child.once('close', () => {
            child = runElectron(browserSyncUrl);
          });
          child.kill();
        });

      bsync
        .watch(['build/src/renderer/**/*', path.resolve('../components/build/**')])
        .on('change', bsync.reload);
    },
  );
}

function prepareWatchArguments(projectPath) {
  return ['--noClear', '--sourceMap', '--project', projectPath];
}

const appWatcher = new TscWatchClient();
const componentsWatcher = new TscWatchClient();

componentsWatcher.on('first_success', () => {
  appWatcher.start(...prepareWatchArguments(path.resolve(__dirname, '..')));
});

appWatcher.on('first_success', () => {
  startBrowserSync();
});

componentsWatcher.start(...prepareWatchArguments(path.resolve(__dirname, '../../components')));
