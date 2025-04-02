const path = require('path');
const fs = require('fs');
const builder = require('electron-builder');
const { Arch } = require('electron-builder');
const { execFileSync } = require('child_process');
const { flipFuses, FuseVersion, FuseV1Options } = require('@electron/fuses');

const noCompression = process.argv.includes('--no-compression');
const shouldNotarize = process.argv.includes('--notarize');

const universal = process.argv.includes('--universal');
const release = process.argv.includes('--release');

const targets = getOptionValue('--targets');
const hostTargetTriple = getOptionValue('--host-target-triple');

function getOptionValue(option) {
  const optionIndex = process.argv.indexOf(option);
  if (optionIndex !== -1) {
    return process.argv[optionIndex + 1];
  }
}

// Adapted from example usages in this Github issue:
// https://github.com/electron-userland/electron-builder/issues/6365
async function flipElectronFuses(context) {
  const { arch, appOutDir, electronPlatformName } = context;

  const fileName = {
    darwin: 'Mullvad VPN.app',
    linux: 'mullvad-vpn',
    win32: 'mullvad-vpn.exe',
  }[electronPlatformName];

  const electronBinaryPath = path.join(appOutDir, fileName);
  const resetAdHocDarwinSignature =
    electronPlatformName === 'darwin' && arch === builder.Arch.arm64; // necessary for building on Apple Silicon

  await flipFuses(electronBinaryPath, {
    version: FuseVersion.V1,
    resetAdHocDarwinSignature,
    [FuseV1Options.EnableNodeCliInspectArguments]: false,
  });
}

function newConfig() {
  return {
    appId: 'net.mullvad.vpn',
    copyright: 'Mullvad VPN AB',
    productName: 'Mullvad VPN',
    publish: null,
    asar: true,
    compression: noCompression ? 'store' : 'normal',
    extraResources: [
      { from: distAssets('ca.crt'), to: '.' },
      { from: buildAssets('relays.json'), to: '.' },
      { from: root('CHANGELOG.md'), to: '.' },
    ],

    directories: {
      buildResources: root('dist-assets'),
      output: root('dist'),
    },

    extraMetadata: {
      name: 'mullvad-vpn',
      // We have to stick to semver on Windows for now due to:
      // https://github.com/electron-userland/electron-builder/issues/7173
      version: productVersion(process.platform === 'win32' ? ['semver'] : []),
    },

    files: [
      'package.json',
      'changes.txt',
      'build/',
      '!**/*.tsbuildinfo',
      '!test/',
      '!playwright.config.ts',
      'node_modules/',
      '!node_modules/grpc-tools',
      '!node_modules/@types',
      '!node_modules/nseventforwarder/debug',
      '!node_modules/windows-utils/debug',
    ],

    // Make sure that all files declared in "extraResources" exists and abort if they don't.
    afterPack: async (context) => {
      const isMac = context.electronPlatformName === 'darwin';
      const isMacUniversal = isMac && context.arch === builder.Arch.universal;

      if (
        // Flip fuses for non-MacOS platforms
        !isMac ||
        // Only flip fuses for universal MacOS package
        isMacUniversal
      ) {
        await flipElectronFuses(context);
      }

      if (context.arch !== Arch.universal) {
        const resources = context.packager.platformSpecificBuildOptions.extraResources;
        for (const resource of resources) {
          const filePath = resource.from.replaceAll(
            /\$\{env\.(.*?)\}/g,
            function (match, captureGroup) {
              return process.env[captureGroup];
            },
          );

          if (!fs.existsSync(filePath)) {
            throw new Error(`Can't find file: ${filePath}`);
          }
        }
      }
    },

    mac: {
      target: {
        target: 'pkg',
        arch: getMacArch(),
      },
      singleArchFiles: 'node_modules/nseventforwarder/dist/**',
      artifactName: 'MullvadVPN-${version}.${ext}',
      category: 'public.app-category.tools',
      icon: distAssets('icon-macos.icns'),
      notarize: shouldNotarize,
      extendInfo: {
        LSUIElement: true,
        NSUserNotificationAlertStyle: 'banner',
      },
      extraResources: [
        { from: distAssets(path.join('${env.BINARIES_PATH}', 'mullvad')), to: '.' },
        { from: distAssets(path.join('${env.BINARIES_PATH}', 'mullvad-problem-report')), to: '.' },
        { from: distAssets(path.join('${env.BINARIES_PATH}', 'mullvad-daemon')), to: '.' },
        { from: distAssets(path.join('${env.BINARIES_PATH}', 'mullvad-setup')), to: '.' },
        {
          from: distAssets(path.join('${env.BINARIES_PATH}', 'libtalpid_openvpn_plugin.dylib')),
          to: '.',
        },
        { from: distAssets(path.join('binaries', '${env.TARGET_TRIPLE}', 'openvpn')), to: '.' },
        { from: distAssets('uninstall_macos.sh'), to: './uninstall.sh' },
        { from: buildAssets('shell-completions/_mullvad'), to: '.' },
        { from: buildAssets('shell-completions/mullvad.fish'), to: '.' },
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
      target: [],
      artifactName: 'MullvadVPN-${version}_${arch}.${ext}',
      publisherName: 'Mullvad VPN AB',
      extraResources: [
        { from: distAssets(path.join('${env.DIST_SUBDIR}', 'mullvad.exe')), to: '.' },
        {
          from: distAssets(path.join('${env.DIST_SUBDIR}', 'mullvad-problem-report.exe')),
          to: '.',
        },
        { from: distAssets(path.join('${env.DIST_SUBDIR}', 'mullvad-daemon.exe')), to: '.' },
        { from: distAssets(path.join('${env.DIST_SUBDIR}', 'talpid_openvpn_plugin.dll')), to: '.' },
        {
          from: root(
            path.join(
              'windows',
              'winfw',
              'bin',
              '${env.TARGET_ARCHITECTURE}-${env.CPP_BUILD_MODE}',
              'winfw.dll',
            ),
          ),
          to: '.',
        },
        // TODO: OpenVPN does not have an ARM64 build yet.
        { from: distAssets('binaries/x86_64-pc-windows-msvc/openvpn.exe'), to: '.' },
        {
          from: distAssets(path.join('binaries', '${env.TARGET_SUBDIR}', 'wintun/wintun.dll')),
          to: '.',
        },
        {
          from: distAssets(
            path.join('binaries', '${env.TARGET_SUBDIR}', 'split-tunnel/mullvad-split-tunnel.sys'),
          ),
          to: '.',
        },
        {
          from: distAssets(
            path.join('binaries', '${env.TARGET_SUBDIR}', 'wireguard-nt/mullvad-wireguard.dll'),
          ),
          to: '.',
        },
        { from: distAssets(path.join('${env.DIST_SUBDIR}', 'libwg.dll')), to: '.' },
        { from: distAssets(path.join('${env.DIST_SUBDIR}', 'maybenot_ffi.dll')), to: '.' },
      ],
    },

    linux: {
      target: [
        {
          target: 'deb',
          arch: getLinuxTargetArch(),
        },
        {
          target: 'rpm',
          arch: getLinuxTargetArch(),
        },
      ],
      artifactName: 'MullvadVPN-${version}_${arch}.${ext}',
      category: 'Network',
      icon: distAssets('icon.icns'),
      extraFiles: [{ from: distAssets('linux/mullvad-gui-launcher.sh'), to: '.' }],
      extraResources: [
        { from: distAssets(path.join(getLinuxTargetSubdir(), 'mullvad-problem-report')), to: '.' },
        { from: distAssets(path.join(getLinuxTargetSubdir(), 'mullvad-setup')), to: '.' },
        {
          from: distAssets(path.join(getLinuxTargetSubdir(), 'libtalpid_openvpn_plugin.so')),
          to: '.',
        },
        { from: distAssets(path.join('linux', 'apparmor_mullvad')), to: '.' },
        { from: distAssets(path.join('binaries', '${env.TARGET_TRIPLE}', 'openvpn')), to: '.' },
      ],
    },

    deb: {
      fpm: [
        '--no-depends',
        '--version',
        getLinuxVersion(),
        '--before-install',
        distAssets('linux/before-install.sh'),
        '--before-remove',
        distAssets('linux/before-remove.sh'),
        distAssets('linux/mullvad-daemon.service') +
          '=/usr/lib/systemd/system/mullvad-daemon.service',
        distAssets('linux/mullvad-early-boot-blocking.service') +
          '=/usr/lib/systemd/system/mullvad-early-boot-blocking.service',
        distAssets(path.join(getLinuxTargetSubdir(), 'mullvad')) + '=/usr/bin/',
        distAssets(path.join(getLinuxTargetSubdir(), 'mullvad-daemon')) + '=/usr/bin/',
        distAssets(path.join(getLinuxTargetSubdir(), 'mullvad-exclude')) + '=/usr/bin/',
        distAssets('linux/problem-report-link') + '=/usr/bin/mullvad-problem-report',
        buildAssets('shell-completions/mullvad.bash') +
          '=/usr/share/bash-completion/completions/mullvad',
        buildAssets('shell-completions/_mullvad') + '=/usr/local/share/zsh/site-functions/_mullvad',
        buildAssets('shell-completions/mullvad.fish') +
          '=/usr/share/fish/vendor_completions.d/mullvad.fish',
      ],
      afterInstall: distAssets('linux/after-install.sh'),
      afterRemove: distAssets('linux/after-remove.sh'),
    },

    rpm: {
      fpm: [
        '--version',
        getLinuxVersion(),
        // Prevents RPM from packaging build-id metadata, some of which is the
        // same across all electron-builder applications, which causes package
        // conflicts
        '--rpm-rpmbuild-define=_build_id_links none',
        '--directories=/opt/Mullvad VPN/',
        '--before-install',
        distAssets('linux/before-install.sh'),
        '--before-remove',
        distAssets('linux/before-remove.sh'),
        '--rpm-posttrans',
        distAssets('linux/post-transaction.sh'),
        distAssets('linux/mullvad-daemon.service') +
          '=/usr/lib/systemd/system/mullvad-daemon.service',
        distAssets('linux/mullvad-early-boot-blocking.service') +
          '=/usr/lib/systemd/system/mullvad-early-boot-blocking.service',
        distAssets(path.join(getLinuxTargetSubdir(), 'mullvad')) + '=/usr/bin/',
        distAssets(path.join(getLinuxTargetSubdir(), 'mullvad-daemon')) + '=/usr/bin/',
        distAssets(path.join(getLinuxTargetSubdir(), 'mullvad-exclude')) + '=/usr/bin/',
        distAssets('linux/problem-report-link') + '=/usr/bin/mullvad-problem-report',
        buildAssets('shell-completions/mullvad.bash') +
          '=/usr/share/bash-completion/completions/mullvad',
        buildAssets('shell-completions/_mullvad') + '=/usr/share/zsh/site-functions/_mullvad',
        buildAssets('shell-completions/mullvad.fish') +
          '=/usr/share/fish/vendor_completions.d/mullvad.fish',
      ],
      afterInstall: distAssets('linux/after-install.sh'),
      afterRemove: distAssets('linux/after-remove.sh'),
      depends: ['libXScrnSaver', 'libnotify', 'dbus-libs'],
    },
  };
}

async function packWin() {
  const DEFAULT_ARCH = targets === 'aarch64-pc-windows-msvc' ? 'arm64' : 'x64';

  function prepareWinConfig(arch) {
    const config = newConfig();
    return {
      ...config,
      win: {
        ...config.win,
        target: [
          {
            target: 'nsis',
            arch: arch,
          },
        ],
      },
      asarUnpack: ['build/assets/images/menubar-icons/win32/lock-*.ico', '**/*.node'],
      beforeBuild: (options) => {
        process.env.CPP_BUILD_MODE = release ? 'Release' : 'Debug';
        process.env.CPP_BUILD_TARGET = options.arch;
        process.env.TARGET_ARCHITECTURE = options.arch;
        switch (options.arch) {
          case 'x64':
            process.env.TARGET_TRIPLE = 'x86_64-pc-windows-msvc';
            process.env.SETUP_SUBDIR = '.';
            process.env.TARGET_SUBDIR = 'x86_64-pc-windows-msvc';
            process.env.DIST_SUBDIR = '';

            execFileSync('npm', ['-w', 'windows-utils', 'run', 'build-x86'], { shell: true });
            break;
          case 'arm64':
            process.env.TARGET_TRIPLE = 'aarch64-pc-windows-msvc';
            process.env.SETUP_SUBDIR = 'aarch64-pc-windows-msvc';
            process.env.TARGET_SUBDIR = 'aarch64-pc-windows-msvc';
            process.env.DIST_SUBDIR = 'aarch64-pc-windows-msvc';

            execFileSync('npm', ['-w', 'windows-utils', 'run', 'build-arm'], { shell: true });
            break;
          default:
            throw new Error('Invalid or unknown target (only one may be specified)');
        }
        return true;
      },
      afterAllArtifactBuild: (buildResult) => {
        // All of this is a hack to work around the limitation in:
        // https://github.com/electron-userland/electron-builder/issues/7173
        const productSemverVersion = productVersion(['semver']);
        const productTargetVersion = productVersion([]);

        // Rename the artifacts so that they don't have the .0 (semver format)
        for (const artifactPath of buildResult.artifactPaths) {
          const artifactDir = path.dirname(artifactPath);
          const artifactSemverFilename = path.basename(artifactPath);
          const artifactDesiredFilename = artifactSemverFilename.replace(
            productSemverVersion,
            productTargetVersion,
          );
          const targetArtifactPath = path.join(artifactDir, artifactDesiredFilename);
          console.log('Moving', artifactSemverFilename, '=>', artifactDesiredFilename);
          fs.renameSync(artifactPath, targetArtifactPath);
        }
      },
    };
  }

  if (universal) {
    // For universal builds, we simply build for all targets. It is up to build.sh to pack the
    // installers in the same binary.
    await builder.build({
      targets: builder.Platform.WINDOWS.createTarget(),
      config: prepareWinConfig(DEFAULT_ARCH === 'x64' ? 'arm64' : 'x64'),
    });
  }

  return builder.build({
    targets: builder.Platform.WINDOWS.createTarget(),
    config: prepareWinConfig(DEFAULT_ARCH),
  });
}

function packMac() {
  const appOutDirs = [];
  const config = newConfig();

  return builder.build({
    targets: builder.Platform.MAC.createTarget(),
    config: {
      ...config,
      asarUnpack: ['**/*.node'],
      beforeBuild: async (options) => {
        switch (options.arch) {
          case 'x64':
            process.env.TARGET_TRIPLE = 'x86_64-apple-darwin';
            execFileSync('npm', ['-w', 'nseventforwarder', 'run', 'build-x86']);
            break;
          case 'arm64':
            process.env.TARGET_TRIPLE = 'aarch64-apple-darwin';
            execFileSync('npm', ['-w', 'nseventforwarder', 'run', 'build-arm']);
            break;
          default:
            delete process.env.TARGET_TRIPLE;
            break;
        }

        process.env.BINARIES_PATH =
          hostTargetTriple !== process.env.TARGET_TRIPLE ? process.env.TARGET_TRIPLE : '';

        return true;
      },
      beforePack: async (context) => {
        await removeNseventforwarderNativeModules();
        config.beforePack?.(context);
      },
      afterPack: async (context) => {
        await config.afterPack?.(context);

        if (context.arch !== Arch.universal) {
          delete process.env.TARGET_TRIPLE;
          appOutDirs.push(context.appOutDir);
        }

        return Promise.resolve();
      },
      afterAllArtifactBuild: async (_buildResult) => {
        // Remove the folder that contains the unpacked app. Electron builder cleans up some of
        // these directories and it's changed between versions without a mention in the changelog.
        for (const dir of appOutDirs) {
          try {
            await fs.promises.rm(dir, { recursive: true });
          } catch {
            // noop
          }
        }
      },
      afterSign: (context) => {
        const appOutDir = context.appOutDir;
        appOutDirs.push(appOutDir);
      },
    },
  });
}

function packLinux() {
  const config = newConfig();

  if (noCompression) {
    config.rpm.fpm.unshift('--rpm-compression', 'none');
  }

  if (targets && targets === 'aarch64-unknown-linux-gnu') {
    config.rpm.fpm.unshift('--architecture', 'aarch64');
  }

  return builder.build({
    targets: builder.Platform.LINUX.createTarget(),
    config: {
      ...config,
      beforeBuild: (options) => {
        switch (options.arch) {
          case 'x64':
            process.env.TARGET_TRIPLE = 'x86_64-unknown-linux-gnu';
            break;
          case 'arm64':
            process.env.TARGET_TRIPLE = 'aarch64-unknown-linux-gnu';
            break;
          default:
            delete process.env.TARGET_TRIPLE;
            break;
        }

        return true;
      },
      afterPack: async (context) => {
        await config.afterPack?.(context);

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

function buildAssets(relativePath) {
  return root(path.join('build', relativePath));
}

function distAssets(relativePath) {
  return root(path.join('dist-assets', relativePath));
}

function root(relativePath) {
  return path.join(path.resolve(__dirname, '../../../../'), relativePath);
}

function getLinuxTargetArch() {
  if (targets && process.platform === 'linux') {
    if (targets === 'aarch64-unknown-linux-gnu') {
      return 'arm64';
    }
    throw new Error('Invalid or unknown target (only one may be specified)');
  }
  // Use host architecture.
  return undefined;
}

function getLinuxTargetSubdir() {
  if (targets && process.platform === 'linux') {
    if (targets === 'aarch64-unknown-linux-gnu') {
      return targets;
    }
    throw new Error('Invalid or unknown target (only one may be specified)');
  }
  return '';
}

function getMacArch() {
  if (universal) {
    return 'universal';
  } else {
    // Not specifying an arch makes Electron builder build for the arch it's running on.
    return undefined;
  }
}

// Replace '-' with `~` (tilde) before the beta component, to make the version comparison
// understand that stable `YYYY.NN` is newer than beta `YYYY.NN-betaN`. Both Debian and
// Fedora do this where a tilde denotes a version component that must be sorted as earlier
// than a non-tilde version component
// https://docs.fedoraproject.org/en-US/packaging-guidelines/Versioning/#_complex_versioning
function getLinuxVersion() {
  const [version, ...prereleaseParts] = productVersion([]).split('-');
  const [major, minor] = version.split('.');
  const prerelease = prereleaseParts.join('-');
  if (prerelease) {
    if (prerelease.toLowerCase().startsWith('beta')) {
      return `${major}.${minor}~${prerelease}`;
    }
    return `${major}.${minor}-${prerelease}`;
  }
  return `${major}.${minor}`;
}

// Returns the product version. The `args` argument is optional. Set it to `'semver'`
// to get the version in semver format.
function productVersion(extraArgs) {
  const args = ['run', '-q', '--bin', 'mullvad-version', ...extraArgs];
  return execFileSync('cargo', args, { encoding: 'utf-8' }).trim();
}

// `@electron/universal` tries to lipo together libraries built for the same architecture
// if they're present for both targets. So make sure we remove libraries for other archs.
// Remove the workaround once the issue has been fixed:
// https://github.com/electron/universal/issues/41#issuecomment-1496288834
//
// dist/darwin-x64/index.node
// dist/darwin-arm64/index.node
async function removeNseventforwarderNativeModules() {
  try {
    await fs.promises.rm('../../node_modules/nseventforwarder/dist/', { recursive: true });
  } catch {
    // noop
  }
}

packWin.displayName = 'builder-win';
packMac.displayName = 'builder-mac';
packLinux.displayName = 'builder-linux';

exports.packWin = packWin;
exports.packMac = packMac;
exports.packLinux = packLinux;
