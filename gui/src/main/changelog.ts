import fs from 'fs';
import path from 'path';
import { IChangelog } from '../shared/ipc-types';
import log from '../shared/logging';

// Reads and parses the changelog file.
export function readChangelog(): IChangelog {
  try {
    const changelogPath = path.join(__dirname, '..', '..', '..', 'changes.txt');
    const contents = fs.readFileSync(changelogPath).toString();
    return parseChangelog(contents);
  } catch (e) {
    const error = e as Error;
    log.error('Failed to read changelog.txt', error.message);
    return [];
  }
}

// Parses the contents of the changelog file and returns all relevant items.
export function parseChangelog(changelog: string): IChangelog {
  const items = changelog
    .split('\n')
    .map((item) => item.trim())
    .filter((item) => item !== '');
  return filterForPlatform(items);
}

// Filters the changelog items based on platform
function filterForPlatform(items: Array<string>): IChangelog {
  return items
    .map((item) => {
      // Extracts the platforms from from the string if there are any specified. Platforms are
      // specified within brackets with separated with a comma.
      const platforms = item
        .match(/^\[.*?\]/)
        ?.flatMap((match) => match.slice(1, -1).split(','))
        .map((platform) => platform.trim());
      if (!platforms || isPlatform(platforms)) {
        // If there are no platforms specified or if the current platform matches one of the
        // specified, then the item is included.
        return item.replace(/^\[.*?\]/, '').trim();
      } else {
        return undefined;
      }
    })
    .filter((item): item is string => item !== undefined);
}

// Checks if an OS name corresponds to the current platform.
function isPlatform(platformNames: Array<string>): boolean {
  const platforms = platformNames.map((platformName) => {
    switch (platformName.toLowerCase()) {
      case 'windows':
        return 'win32';
      case 'macos':
        return 'darwin';
      case 'linux':
        return 'linux';
      default:
        return platformName;
    }
  });

  return platforms.includes(process.platform);
}
