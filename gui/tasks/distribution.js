const path = require('path');
const fs = require('fs');
const builder = require('electron-builder');
const parseSemver = require('semver/functions/parse');
const { notarize } = require('electron-notarize');
const { version } = require('../package.json');

const noCompression = process.argv.includes('--no-compression');
const noAppleNotarization = process.argv.includes('--no-apple-notarization');

const universal = process.argv.includes('--universal');
const release = process.argv.includes('--release');

const config = {
  appId: 'net.mullvad.vpn',
  copyright: 'Mullvad VPN AB',
  productName: 'Mullvad VPN',
  asar: true,
  compression: noCompression ? 'store' : 'normal',
  extraResources: [
    { from: distAssets('ca.crt'), to: '.' },
    { from: distAssets('relays.json'), to: '.' },
    { from: root('CHANGELOG.md'), to: '.' },
  ],

  directories: {
    buildResources: root('dist-assets'),
    output: root('dist'),
  },

  extraMetadata: {
    name: 'mullvad-vpn',
  },

  files: [
    'package.json',
    'changes.txt',
    'init.js',
    'build/',
    '!build/src/renderer',
    'build/src/renderer/index.html',
    'build/src/renderer/bundle.js',
    'build/src/renderer/preloadBundle.js',
    '!**/*.tsbuildinfo',
    'node_modules/',
    '!node_modules/grpc-tools',
    '!node_modules/@types',
  ],

  // Make sure that all files declared in "extraResources" exists and abort if they don't.
  afterPack: (context) => {
    const resources = context.packager.platformSpecificBuildOptions.extraResources;
    for (const resource of resources) {
      const filePath = resource.from.replace(/\$\{env\.(.*)\}/, function (match, captureGroup) {
        return process.env[captureGroup];
      });

      if (!fs.existsSync(filePath)) {
        throw new Error(`Can't find file: ${filePath}`);
      }
    }
  },

  mac: {
    target: {
      target: 'pkg',
      arch: getMacArch(),
    },
    singleArchFiles: 'node_modules/nseventmonitor/lib/binding/Release/**',
    artifactName: 'MullvadVPN-${version}.${ext}',
    category: 'public.app-category.tools',
    icon: distAssets('icon-macos.icns'),
    extendInfo: {
      LSUIElement: true,
      NSUserNotificationAlertStyle: 'alert',
    },
    extraResources: [
      { from: distAssets(path.join('${env.TARGET_TRIPLE}', 'mullvad')), to: '.' },
      { from: distAssets(path.join('${env.TARGET_TRIPLE}', 'mullvad-problem-report')), to: '.' },
      { from: distAssets(path.join('${env.TARGET_TRIPLE}', 'mullvad-daemon')), to: '.' },
      { from: distAssets(path.join('${env.TARGET_TRIPLE}', 'mullvad-setup')), to: '.' },
      {
        from: distAssets(path.join('${env.TARGET_TRIPLE}', 'libtalpid_openvpn_plugin.dylib')),
        to: '.',
      },
      { from: distAssets(path.join('binaries', '${env.TARGET_TRIPLE}', 'openvpn')), to: '.' },
      { from: distAssets('uninstall_macos.sh'), to: './uninstall.sh' },
      { from: distAssets('shell-completions/_mullvad'), to: '.' },
      { from: distAssets('shell-completions/mullvad.fish'), to: '.' },
    ],
  },

  pkg: {
    allowAnywhere: false,
    allowCurrentUserHome: false,
    isRelocatable: false,
    isVersionChecked: false,
  },

  nsis: {
    guid: '2A356FD4-03B7-4F45-99B4-737BE580DC82',
    oneClick: false,
    perMachine: true,
    allowElevation: true,
    allowToChangeInstallationDirectory: false,
    include: distAssets('windows/installer.nsh'),
    installerSidebar: distAssets('windows/installersidebar.bmp'),
  },

  win: {
    target: [
      {
        target: 'nsis',
        arch: ['x64'],
      },
    ],
    artifactName: 'MullvadVPN-${version}.${ext}',
    publisherName: 'Mullvad VPN AB',
    signingHashAlgorithms: ['sha256'],
    signDlls: true,
    extraResources: [
      { from: distAssets('mullvad.exe'), to: '.' },
      { from: distAssets('mullvad-problem-report.exe'), to: '.' },
      { from: distAssets('mullvad-daemon.exe'), to: '.' },
      { from: distAssets('talpid_openvpn_plugin.dll'), to: '.' },
      {
        from: root(path.join('windows', 'winfw', 'bin', 'x64-${env.CPP_BUILD_MODE}', 'winfw.dll')),
        to: '.',
      },
      {
        from: root(
          path.join('windows', 'windns', 'bin', 'x64-${env.CPP_BUILD_MODE}', 'windns.dll'),
        ),
        to: '.',
      },
      {
        from: root(
          path.join('windows', 'winnet', 'bin', 'x64-${env.CPP_BUILD_MODE}', 'winnet.dll'),
        ),
        to: '.',
      },
      { from: distAssets('binaries/x86_64-pc-windows-msvc/openvpn.exe'), to: '.' },
      { from: root('build/lib/x86_64-pc-windows-msvc/libwg.dll'), to: '.' },
      { from: distAssets('binaries/x86_64-pc-windows-msvc/wintun/wintun.dll'), to: '.' },
      {
        from: distAssets('binaries/x86_64-pc-windows-msvc/wireguard-nt/mullvad-wireguard.dll'),
        to: '.',
      },
    ],
  },

  linux: {
    target: ['deb', 'rpm'],
    artifactName: 'MullvadVPN-${version}_${arch}.${ext}',
    category: 'Network',
    icon: distAssets('icon.icns'),
    extraFiles: [{ from: distAssets('linux/mullvad-gui-launcher.sh'), to: '.' }],
    extraResources: [
      { from: distAssets('mullvad-problem-report'), to: '.' },
      { from: distAssets('mullvad-daemon'), to: '.' },
      { from: distAssets('mullvad-setup'), to: '.' },
      { from: distAssets('libtalpid_openvpn_plugin.so'), to: '.' },
      { from: distAssets('binaries/x86_64-unknown-linux-gnu/openvpn'), to: '.' },
      { from: distAssets('linux/mullvad-daemon.conf'), to: '.' },
      { from: distAssets('linux/mullvad-daemon.service'), to: '.' },
    ],
  },

  deb: {
    fpm: [
      '--no-depends',
      '--version',
      getDebVersion(),
      '--before-install',
      distAssets('linux/before-install.sh'),
      '--before-remove',
      distAssets('linux/before-remove.sh'),
      '--config-files',
      '/opt/Mullvad VPN/resources/mullvad-daemon.service',
      '--config-files',
      '/opt/Mullvad VPN/resources/mullvad-daemon.conf',
      distAssets('mullvad') + '=/usr/bin/',
      distAssets('mullvad-exclude') + '=/usr/bin/',
      distAssets('linux/problem-report-link') + '=/usr/bin/mullvad-problem-report',
      distAssets('shell-completions/mullvad.bash') +
        '=/usr/share/bash-completion/completions/mullvad',
      distAssets('shell-completions/_mullvad') + '=/usr/local/share/zsh/site-functions/_mullvad',
      distAssets('shell-completions/mullvad.fish') +
        '=/usr/share/fish/vendor_completions.d/mullvad.fish',
    ],
    afterInstall: distAssets('linux/after-install.sh'),
    afterRemove: distAssets('linux/after-remove.sh'),
  },

  rpm: {
    fpm: [
      '--before-install',
      distAssets('linux/before-install.sh'),
      '--before-remove',
      distAssets('linux/before-remove.sh'),
      '--rpm-posttrans',
      distAssets('linux/post-transaction.sh'),
      '--config-files',
      '/opt/Mullvad VPN/resources/mullvad-daemon.service',
      '--config-files',
      '/opt/Mullvad VPN/resources/mullvad-daemon.conf',
      distAssets('mullvad') + '=/usr/bin/',
      distAssets('mullvad-exclude') + '=/usr/bin/',
      distAssets('linux/problem-report-link') + '=/usr/bin/mullvad-problem-report',
      distAssets('shell-completions/mullvad.bash') +
        '=/usr/share/bash-completion/completions/mullvad',
      distAssets('shell-completions/_mullvad') + '=/usr/share/zsh/site-functions/_mullvad',
      distAssets('shell-completions/mullvad.fish') +
        '=/usr/share/fish/vendor_completions.d/mullvad.fish',
    ],
    afterInstall: distAssets('linux/after-install.sh'),
    afterRemove: distAssets('linux/after-remove.sh'),
    depends: ['libXScrnSaver', 'libnotify', 'libnsl', 'dbus-libs'],
  },
};

function packWin() {
  return builder.build({
    targets: builder.Platform.WINDOWS.createTarget(),
    config: {
      ...config,
      asarUnpack: ['build/assets/images/menubar icons/win32/lock-*.ico'],
      beforeBuild: (options) => {
        process.env.CPP_BUILD_MODE = release ? 'Release' : 'Debug';
        return true;
      },
    },
  });
}

function packMac() {
  const appOutDirs = [];

  return builder.build({
    targets: builder.Platform.MAC.createTarget(),
    config: {
      ...config,
      asarUnpack: ['**/*.node'],
      beforeBuild: (options) => {
        switch (options.arch) {
          case 'x64':
            process.env.TARGET_TRIPLE = 'x86_64-apple-darwin';
            break;
          case 'arm64':
            process.env.TARGET_TRIPLE = 'aarch64-apple-darwin';
            break;
          default:
            delete process.env.TARGET_TRIPLE;
            break;
        }

        return true;
      },
      afterPack: (context) => {
        config.afterPack?.(context);

        delete process.env.TARGET_TRIPLE;
        appOutDirs.push(context.appOutDir);
        return Promise.resolve();
      },
      afterAllArtifactBuild: async (buildResult) => {
        if (!noAppleNotarization) {
          // buildResult.artifactPaths[0] contains the path to the pkg.
          await notarizeMac(buildResult.artifactPaths[0]);
        }

        // Remove the folder that contains the unpacked app. Electron builder cleans up some of
        // these directories and it's changed between versions without a mention in the changelog.
        for (const dir of appOutDirs) {
          try {
            await fs.promises.rm(dir, { recursive: true });
          } catch {}
        }
      },
      afterSign: (context) => {
        const appOutDir = context.appOutDir;
        appOutDirs.push(appOutDir);

        if (!noAppleNotarization) {
          const appName = context.packager.appInfo.productFilename;
          return notarizeMac(path.join(appOutDir, `${appName}.app`));
        }
      },
    },
  });
}

function notarizeMac(notarizePath) {
  console.log('Notarizing ' + notarizePath);
  return notarize({
    appBundleId: config.appId,
    appPath: notarizePath,
    appleId: process.env.NOTARIZE_APPLE_ID,
    appleIdPassword: process.env.NOTARIZE_APPLE_ID_PASSWORD,
  });
}

function packLinux() {
  if (noCompression) {
    config.rpm.fpm.unshift('--rpm-compression', 'none');
  }

  return builder.build({
    targets: builder.Platform.LINUX.createTarget(),
    config: {
      ...config,
      afterPack: async (context) => {
        config.afterPack?.(context);

        const sourceExecutable = path.join(context.appOutDir, 'mullvad-vpn');
        const targetExecutable = path.join(context.appOutDir, 'mullvad-gui');
        const launcherScript = path.join(context.appOutDir, 'mullvad-gui-launcher.sh');

        // rename mullvad-vpn to mullvad-gui
        await fs.promises.rename(sourceExecutable, targetExecutable);
        // rename launcher script to mullvad-vpn
        await fs.promises.rename(launcherScript, sourceExecutable);
      },
    },
  });
}

function distAssets(relativePath) {
  return path.join(path.resolve(__dirname, '../../dist-assets'), relativePath);
}

function root(relativePath) {
  return path.join(path.resolve(__dirname, '../../'), relativePath);
}

function getMacArch() {
  if (universal) {
    return 'universal';
  } else {
    // Not specifying an arch makes Electron builder build for the arch it's running on.
    return undefined;
  }
}

// Replace '-' between components with a tilde to make the version comparison understand that
// YYYY.NN > YYYY.NN-betaN > YYYY.NN-betaN-dev-HHHHHH.
function getDebVersion() {
  const { major, minor, prerelease } = parseSemver(version);
  const versionParts = [`${major}.${minor}`];
  if (prerelease[0]) {
    // Replace first '-' with a '~' since the first one is the one between 'betaN' and 'dev-hash'.
    versionParts.push(prerelease[0].replace('-', '~'));
  }
  return versionParts.join('~');
}

packWin.displayName = 'builder-win';
packMac.displayName = 'builder-mac';
packLinux.displayName = 'builder-linux';

exports.packWin = packWin;
exports.packMac = packMac;
exports.packLinux = packLinux;
