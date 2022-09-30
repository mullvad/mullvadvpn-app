import child_process from 'child_process';
import { nativeImage } from 'electron';
import fs from 'fs';
import path from 'path';

import { ILinuxApplication } from '../shared/application-types';
import log from '../shared/logging';

type DirectoryDescription = string | RegExp;

export interface DesktopEntry {
  absolutepath: string;
  name: string;
  type: string;
  icon?: string;
  exec?: string;
  terminal?: string;
  noDisplay?: string;
  hidden?: string;
  onlyShowIn?: string[];
  notShowIn?: string[];
  tryExec?: string;
}

const DESKTOP_ENTRY_KEYS = [
  'name',
  'type',
  'icon',
  'exec',
  'terminal',
  'noDisplay',
  'hidden',
  'onlyShowIn',
  'notShowIn',
  'tryExec',
];

const LIST_KEYS = ['onlyShowIn', 'notShowIn'];

// Parses a desktop entry at a specific path. Implemented in accordance with the freedesktop.org's
// Desktop Entry Specification:
// https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html
export async function readDesktopEntry(entryPath: string, locale?: string): Promise<DesktopEntry> {
  // First the lines corresponding to desktop entry group is extracted from the file
  const contents = (await fs.promises.readFile(entryPath)).toString().split('\n');
  // The group start is indicated by `[Desktop Entry]`
  const startIndex = contents.indexOf('[Desktop Entry]') + 1;
  const contentsFromDesktopEntry = contents.slice(startIndex);
  // The group ens when the next group start
  const endIndex = contentsFromDesktopEntry.findIndex((line) => /^\[.*\]$/.test(line));
  const desktopEntry = contentsFromDesktopEntry.slice(0, endIndex);

  return parseDesktopEntry(entryPath, desktopEntry, locale);
}

// Parses the values within the desktop entry group in a desktop entry file
function parseDesktopEntry(
  absolutepath: string,
  desktopEntry: string[],
  locale?: string,
): DesktopEntry {
  const parsed: Partial<DesktopEntry> = desktopEntry.reduce(
    (entry, line) => parseDesktopEntryLine(entry, line, locale),
    { absolutepath } as Partial<DesktopEntry>,
  );

  // If the dekstop entry is lacking some of the required keys it's invalid
  if (isDesktopEntry(parsed)) {
    return parsed;
  } else {
    throw new Error('Not a correctly formatted desktop entry');
  }
}

// Parses a line in a desktop entry
function parseDesktopEntryLine(
  entry: Partial<DesktopEntry>,
  line: string,
  locale?: string,
): Partial<DesktopEntry> {
  // Comments start with `#` and keys and values are seperated by a `=`
  if (!line.startsWith('#') && line.includes('=')) {
    const firstEqualSign = line.indexOf('=');
    const keyWithLocale = line.slice(0, firstEqualSign).replace(' ', '');
    const value = line.slice(firstEqualSign + 1).trim();

    // Key values can be suffixed by a locale enclosed in `[]`
    const pascalCaseKey = keyWithLocale.replace(/\[.*\]/, '');
    const key = pascalCaseKey[0].toLowerCase() + pascalCaseKey.slice(1);
    const keyLocale = keyWithLocale.match(/\[.*\]/)?.[0].replace(/(\[|\])/g, '');

    // If the key locale match the provided locale the value is used, otherwise it's only used if
    // there isn't a value already
    if (isDesktopEntryKey(key) && (keyLocale ? keyLocale === locale : entry[key] === undefined)) {
      // Some values are lists of values and they have to be split on `;` and ofter contain a
      // trailing `;`
      if (LIST_KEYS.includes(key)) {
        const arrayValue = value.replace(/;$/, '').split(';');
        return { ...entry, [key]: arrayValue };
      } else {
        return { ...entry, [key]: value };
      }
    }
  }

  return entry;
}

function isDesktopEntryKey(key: string): key is keyof DesktopEntry {
  return DESKTOP_ENTRY_KEYS.includes(key);
}

function isDesktopEntry(entry: Partial<DesktopEntry>): entry is DesktopEntry {
  return entry.absolutepath !== undefined && entry.name !== undefined && entry.type !== undefined;
}

// Scans for desktop entries in accordance with the Desktop Entry Specification
export async function getDesktopEntries(): Promise<string[]> {
  const directories = getDesktopEntryDirectories();

  const entries = await directories.reduce(
    async (entries, directory) => getDesktopEntriesInDirectory(directory, await entries),
    Promise.resolve({}),
  );

  return Object.values(entries);
}

async function getDesktopEntriesInDirectory(
  directory: string,
  previousEntries: { [id: string]: string },
  prefix = '',
): Promise<{ [id: string]: string }> {
  let currentEntries = { ...previousEntries };
  try {
    const contents = await fs.promises.readdir(directory);

    for (const item of contents) {
      const id = prefix + item;
      if (path.extname(item) === '.desktop') {
        if (currentEntries[id] === undefined) {
          currentEntries[id] = path.join(directory, item);
        }
      } else {
        const nextDirectory = path.join(directory, item);
        currentEntries = await getDesktopEntriesInDirectory(
          nextDirectory,
          currentEntries,
          `${prefix}${item}-`,
        );
      }
    }
  } catch {
    // no-op
  }

  return currentEntries;
}

function getDesktopEntryDirectories() {
  const directories: string[] = [];

  if (process.env.HOME) {
    directories.push(path.join(process.env.HOME, '.local', 'share', 'applications'));
  }

  const xdgDataDirs = getXdgDataDirs().map((dir) => path.join(dir, 'applications'));
  directories.push(...xdgDataDirs);

  return directories;
}

// Implemented according to freedesktop specification
// https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html
// TODO: Respect "TryExec"
export function shouldShowApplication(application: DesktopEntry): application is ILinuxApplication {
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
export async function findIconPath(
  name: string,
  allowedExtensions = ['svg', 'png'],
): Promise<string | undefined> {
  // Chromium doesn't support .xpm files
  return findIcon(name, allowedExtensions, [
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

  const xdgDataDirs = getXdgDataDirs().map((dir) => path.join(dir, 'icons'));
  directories.push(...xdgDataDirs);
  directories.push('/usr/share/pixmaps');

  return directories;
}

function getXdgDataDirs(): string[] {
  return process.env.XDG_DATA_DIRS?.split(':') ?? ['/usr/local/share/', '/usr/share/'];
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
    } catch (e) {
      const error = e as NodeJS.ErrnoException;
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
