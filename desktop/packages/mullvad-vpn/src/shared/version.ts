import { Url, urls } from './constants';

export function getDownloadUrl(suggestedIsBeta: boolean): Url {
  let url: Url = urls.download;
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

  return url as Url;
}
