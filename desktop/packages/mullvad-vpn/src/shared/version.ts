import { links } from '../config.json';

export function getDownloadUrl(suggestedIsBeta: boolean): string {
  let url = links.download;
  switch (process.platform ?? window.env.platform) {
    case 'win32':
      url += 'windows/';
      break;
    case 'linux':
      url += 'linux/';
      break;
    case 'darwin':
      url += 'macos/';
      break;
  }

  if (suggestedIsBeta) {
    url += 'beta/';
  }

  return url;
}
