import { Link, links } from './constants';

export function getDownloadUrl(suggestedIsBeta: boolean): Link {
  let url: Link = links.download;
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

  return url as Link;
}
