const path = require('path');
const fs = require('fs');
const builder = require('electron-builder');
const rimraf = require('rimraf');
const util = require('util');
const { notarize } = require('electron-notarize');

const renameAsync = util.promisify(fs.rename);
const unlinkAsync = util.promisify(fs.unlink);
const rimrafAsync = util.promisify(rimraf);

const compression = process.argv.indexOf('--no-compression') !== -1 ? 'store' : 'normal';
const noAppleNotarization = process.argv.indexOf('--no-apple-notarization') !== -1;

const config = {
  appId: 'net.mullvad.vpn',
  copyright: 'Amagicom AB',
  productName: 'Mullvad VPN',
  asar: true,
  compression: compression,
  extraResources: [
    { from: distAssets('ca.crt'), to: '.' },
    { from: distAssets('api_root_ca.pem'), to: '.' },
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

  files: ['package.json', 'init.js', 'build/', 'node_modules/', '!**/*.tsbuildinfo'],

  mac: {
    target: 'pkg',
    artifactName: 'MullvadVPN-${version}.${ext}',
    category: 'public.app-category.tools',
    extendInfo: {
      LSUIElement: true,
      NSUserNotificationAlertStyle: 'alert',
    },
    extraResources: [
      { from: distAssets('mullvad'), to: '.' },
      { from: distAssets('mullvad-problem-report'), to: '.' },
      { from: distAssets('mullvad-daemon'), to: '.' },
      { from: distAssets('libtalpid_openvpn_plugin.dylib'), to: '.' },
      { from: distAssets('binaries/x86_64-apple-darwin/openvpn'), to: '.' },
      { from: distAssets('binaries/x86_64-apple-darwin/sslocal'), to: '.' },
      { from: distAssets('uninstall_macos.sh'), to: './uninstall.sh' },
    ],
  },

  pkg: {
    allowAnywhere: false,
    allowCurrentUserHome: false,
    isRelocatable: false,
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
    publisherName: 'Amagicom AB',
    signingHashAlgorithms: ['sha256'],
    signDlls: true,
    extraResources: [
      { from: distAssets('mullvad.exe'), to: '.' },
      { from: distAssets('mullvad-problem-report.exe'), to: '.' },
      { from: distAssets('mullvad-daemon.exe'), to: '.' },
      { from: distAssets('talpid_openvpn_plugin.dll'), to: '.' },
      { from: root('windows/winfw/bin/x64-Release/winfw.dll'), to: '.' },
      { from: root('windows/windns/bin/x64-Release/windns.dll'), to: '.' },
      { from: root('windows/winnet/bin/x64-Release/winnet.dll'), to: '.' },
      { from: root('windows/winutil/bin/x64-Release/winutil.dll'), to: '.' },
      { from: distAssets('binaries/x86_64-pc-windows-msvc/openvpn.exe'), to: '.' },
      { from: distAssets('binaries/x86_64-pc-windows-msvc/sslocal.exe'), to: '.' },
      { from: root('build/lib/x86_64-pc-windows-msvc/libwg.dll'), to: '.' },
    ],
  },

  linux: {
    target: ['deb', 'rpm'],
    artifactName: 'MullvadVPN-${version}_${arch}.${ext}',
    category: 'Network',
    extraFiles: [{ from: distAssets('linux/mullvad-gui-launcher.sh'), to: '.' }],
    extraResources: [
      { from: distAssets('mullvad-problem-report'), to: '.' },
      { from: distAssets('mullvad-daemon'), to: '.' },
      { from: distAssets('libtalpid_openvpn_plugin.so'), to: '.' },
      { from: distAssets('binaries/x86_64-unknown-linux-gnu/openvpn'), to: '.' },
      { from: distAssets('binaries/x86_64-unknown-linux-gnu/sslocal'), to: '.' },
      { from: distAssets('linux/mullvad-daemon.conf'), to: '.' },
      { from: distAssets('linux/mullvad-daemon.service'), to: '.' },
    ],
  },

  deb: {
    fpm: [
      '--before-install',
      distAssets('linux/before-install.sh'),
      '--before-remove',
      distAssets('linux/before-remove.sh'),
      '--config-files',
      '/opt/Mullvad VPN/resources/mullvad-daemon.service',
      '--config-files',
      '/opt/Mullvad VPN/resources/mullvad-daemon.conf',
      distAssets('mullvad') + '=/usr/bin/',
      distAssets('linux/problem-report-link') + '=/usr/bin/mullvad-problem-report',
    ],
    afterInstall: distAssets('linux/after-install.sh'),
    afterRemove: distAssets('linux/after-remove.sh'),
    depends: ['iputils-ping'],
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
      distAssets('linux/problem-report-link') + '=/usr/bin/mullvad-problem-report',
    ],
    afterInstall: distAssets('linux/after-install.sh'),
    afterRemove: distAssets('linux/after-remove.sh'),
    depends: ['libXScrnSaver', 'libnotify', 'libnsl', 'dbus-libs'],
  },
};

function packWin() {
  return builder.build({
    targets: builder.Platform.WINDOWS.createTarget(),
    config: config,
  });
}

function packMac() {
  let appOutDir;

  return builder.build({
    targets: builder.Platform.MAC.createTarget(),
    config: {
      ...config,
      afterPack: (context) => {
        appOutDir = context.appOutDir;
        return Promise.resolve();
      },
      afterAllArtifactBuild: async (buildResult) => {
        if (!noAppleNotarization) {
          await notarizeMac(buildResult.artifactPaths[0]);
        }
        // remove the folder that contains the unpacked app
        return rimrafAsync(appOutDir);
      },
      afterSign: noAppleNotarization
        ? undefined
        : (context) => {
            const appOutDir = context.appOutDir;
            const appName = context.packager.appInfo.productFilename;
            return notarizeMac(path.join(appOutDir, `${appName}.app`));
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
  return builder.build({
    targets: builder.Platform.LINUX.createTarget(),
    config: {
      ...config,
      afterPack: (context) => {
        const sourceExecutable = path.join(context.appOutDir, 'mullvad-vpn');
        const targetExecutable = path.join(context.appOutDir, 'mullvad-gui');
        const launcherScript = path.join(context.appOutDir, 'mullvad-gui-launcher.sh');
        const chromeSandbox = path.join(context.appOutDir, 'chrome-sandbox');

        return Promise.all([
          // rename mullvad-vpn to mullvad-gui
          renameAsync(sourceExecutable, targetExecutable),

          // rename launcher script to mullvad-vpn
          renameAsync(launcherScript, sourceExecutable),

          // remove the chrome-sandbox file since we explicitly disable it
          unlinkAsync(chromeSandbox),
        ]);
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

packWin.displayName = 'builder-win';
packMac.displayName = 'builder-mac';
packLinux.displayName = 'builder-linux';

exports.packWin = packWin;
exports.packMac = packMac;
exports.packLinux = packLinux;
