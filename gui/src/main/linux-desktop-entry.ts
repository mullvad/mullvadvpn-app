import child_process from 'child_process';
import { nativeImage } from 'electron';
import fs from 'fs';
import path from 'path';
import { ILinuxApplication } from '../shared/application-types';
import log from '../shared/logging';

type DirectoryDescription = string | RegExp;

// Implemented according to freedesktop specification
// https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html
// TODO: Respect "TryExec"
export function shouldShowApplication(application: ILinuxApplication): boolean {
  const originalXdgCurrentDesktop = process.env.ORIGINAL_XDG_CURRENT_DESKTOP?.split(':') ?? [];
  const xdgCurrentDesktop = process.env.XDG_CURRENT_DESKTOP?.split(':') ?? [];
  const desktopEnvironments = originalXdgCurrentDesktop.concat(xdgCurrentDesktop);

  const notShowIn =
    typeof application.notShowIn === 'string' ? [application.notShowIn] : application.notShowIn;
  const onlyShowIn =
    typeof application.onlyShowIn === 'string' ? [application.onlyShowIn] : application.onlyShowIn;

  const notShowInMatch = notShowIn?.some((desktopEnvironment) =>
    desktopEnvironments?.includes(desktopEnvironment),
  );
  const onlyShowInMatch =
    onlyShowIn?.some((desktopEnvironment) => desktopEnvironments?.includes(desktopEnvironment)) ??
    false;

  return (
    application.type === 'Application' &&
    application.name !== 'Mullvad VPN' &&
    application.exec !== undefined &&
    application.noDisplay !== 'true' &&
    application.terminal !== 'true' &&
    application.hidden !== 'true' &&
    !notShowInMatch &&
    (!application.onlyShowIn || onlyShowInMatch)
  );
}

export async function getImageDataUrl(imagePath: string): Promise<string> {
  if (imagePath && path.extname(imagePath) === '.svg') {
    const contents = await fs.promises.readFile(imagePath);
    return `data:image/svg+xml;base64,${contents.toString('base64')}`;
  } else {
    const image = nativeImage.createFromPath(imagePath);

    if (image.isEmpty()) {
      log.error(`Failed to load nativeImage: ${imagePath}`);
      throw new Error(`Failed to load nativeImage: ${imagePath}`);
    } else {
      return image.toDataURL();
    }
  }
}

// Returns the path of the icon with the specified name. If none is found it returns undefined.
export async function findIconPath(name: string): Promise<string | undefined> {
  // Chromium doesn't support .xpm files
  const extensions = ['svg', 'png'];
  return findIcon(name, extensions, [
    getIconDirectories(),
    await getGtkThemeDirectories(),
    // Begin with preferred sized but if nothing matches other sizes should be considered as well.
    ['scalable', '256x256', '512x512', '256x256@2x', '128x128@2x', '128x128', /^\d+x\d+(@2x)?$/],
    // Search in all categories of icons.
    [/.*/],
  ]);
}

// Implemented according to freedesktop specification.
// https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html
function getIconDirectories() {
  const directories: string[] = [];

  if (process.env.HOME) {
    directories.push(path.join(process.env.HOME, '.icons'));
    directories.push(path.join(process.env.HOME, '.local', 'share', 'icons')); // For KDE Plasma
  }

  if (process.env.XDG_DATA_DIRS) {
    const dataDirs = process.env.XDG_DATA_DIRS.split(':').map((dir) => path.join(dir, 'icons'));
    directories.push(...dataDirs);
  }

  directories.push('/usr/share/pixmaps');

  return directories;
}

function getGtkThemeDirectories(): Promise<DirectoryDescription[]> {
  // "hicolor" is fallback theme and should always be checked. If no icon is found search is
  // continued in other themes.
  const themes = ['hicolor', /.*/];
  return new Promise((resolve, _reject) => {
    // Electron modifies XDG_CURRENT_DESKTOP and saves the old value in ORIGINAL_XDG_CURRENT_DESKTOP
    const xdgCurrentDesktop =
      process.env.ORIGINAL_XDG_CURRENT_DESKTOP ?? process.env.XDG_CURRENT_DESKTOP ?? '';
    child_process.exec(
      'gsettings get org.gnome.desktop.interface icon-theme',
      // eslint-disable-next-line @typescript-eslint/naming-convention
      { env: { XDG_CURRENT_DESKTOP: xdgCurrentDesktop } },
      (error, stdout) => {
        if (error) {
          log.error('Error while retrieving theme', error);
          resolve(themes);
        } else {
          const theme = stdout.trim().replace(new RegExp("^'|'$", 'g'), '');
          resolve(theme === '' ? themes : [theme, ...themes]);
        }
      },
    );
  });
}

// Searches through a directory tree according to the directory lists supplied. E.g. The arguments
// ('mullvad', ['svg', 'png'], [['a', 'b'], ['c', 'd']]) will search for mullvad.svg and mullvad.png
// in the directories a, a/c, a/d, b, b/c and b/d.
async function findIcon(
  name: string,
  extensions: string[],
  [directories, ...restDirectories]: [string[], ...DirectoryDescription[][]],
): Promise<string | undefined> {
  for (const directory of directories) {
    let contents: string[] | undefined;
    try {
      contents = await fs.promises.readdir(directory);
    } catch (error) {
      // Non-existent directories and files (not a directory) are expected.
      if (error.code !== 'ENOENT' && error.code !== 'ENOTDIR') {
        log.error(`Failed to open directory while searching for ${name} icon`, error);
      }
    }

    if (contents) {
      const iconPath = contents.find((item) =>
        extensions.some((extension) => item === `${name}.${extension}`),
      );

      if (iconPath) {
        return path.join(directory, iconPath);
      } else if (restDirectories.length > 0) {
        const nextDirectories = matchDirectories(restDirectories[0], contents);
        const iconPath = await findIcon(name, extensions, [
          nextDirectories.map((nextDirectory) => path.join(directory, nextDirectory)),
          ...restDirectories.slice(1),
        ]);

        if (iconPath) {
          return iconPath;
        }
      }
    }
  }

  return undefined;
}

function matchDirectories(directories: DirectoryDescription[], contents: string[]) {
  const matches = directories
    .map((directory) =>
      directory instanceof RegExp ? contents.filter((item) => directory.test(item)) : directory,
    )
    .flat();

  // Remove duplicates
  return [...new Set(matches)];
}
