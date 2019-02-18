import path from 'path';

export function resolveBin(binaryName: string) {
  return path.resolve(getBasePath(), binaryName + getExtension());
}

function getBasePath(): string {
  if (process.env.NODE_ENV === 'development') {
    return (
      process.env.MULLVAD_PATH ||
      path.resolve(path.join(__dirname, '../../../../../../target/debug'))
    );
  } else {
    return process.resourcesPath!;
  }
}

function getExtension() {
  switch (process.platform) {
    case 'win32':
      return '.exe';

    default:
      return '';
  }
}
