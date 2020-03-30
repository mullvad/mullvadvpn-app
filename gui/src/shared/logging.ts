import { app, remote } from 'electron';
import log from 'electron-log';
import * as fs from 'fs';
import * as path from 'path';

const LOG_FORMAT = '[{y}-{m}-{d} {h}:{i}:{s}.{ms}][{level}] {text}';

// Returns platform specific logs folder for application
// See open issue and PR on Github:
// 1. https://github.com/electron/electron/issues/10118
// 2. https://github.com/electron/electron/pull/10191
export function getLogsDirectory() {
  const theApp = process.type === 'browser' ? app : remote.app;

  switch (process.platform) {
    case 'darwin':
      // macOS: ~/Library/Logs/{appname}
      return path.join(theApp.getPath('home'), 'Library/Logs', theApp.name);
    default:
      // Windows: %LOCALAPPDATA%\{appname}\logs
      // Linux: ~/.config/{appname}/logs
      return path.join(theApp.getPath('userData'), 'logs');
  }
}

export function getMainLogFile(): string {
  return path.join(getLogsDirectory(), 'frontend.log');
}

export function getRendererLogFile(): string {
  return path.join(getLogsDirectory(), 'frontend-renderer.log');
}

export function setupLogging(logFile: string) {
  log.transports.console.format = LOG_FORMAT;
  log.transports.file.format = LOG_FORMAT;
  log.transports.console.level = 'debug';

  if (process.env.NODE_ENV === 'development') {
    // Disable log file in development
    log.transports.file.level = false;
  } else {
    // Configure logging to file
    log.transports.file.level = 'debug';
    log.transports.file.resolvePath = (_variables) => logFile;

    log.debug(`Logging to ${logFile}`);
  }
}

export function backupLogFile(filePath: string): string | undefined {
  const ext = path.extname(filePath);
  const baseName = path.basename(filePath, ext);
  const backupFile = path.join(path.dirname(filePath), baseName + '.old' + ext);

  try {
    fs.accessSync(filePath);
    fs.renameSync(filePath, backupFile);

    return backupFile;
  } catch (error) {
    // No previous log file exists
    return undefined;
  }
}
