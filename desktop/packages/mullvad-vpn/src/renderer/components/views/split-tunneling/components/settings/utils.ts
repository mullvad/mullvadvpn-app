export function getFilePickerOptionsForPlatform():
  | { name: string; extensions: Array<string> }
  | undefined {
  return window.env.platform === 'win32'
    ? { name: 'Executables', extensions: ['exe', 'lnk'] }
    : undefined;
}
