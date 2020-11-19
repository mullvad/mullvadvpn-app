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

export function backupLogFile(filePath: string): boolean {
  const exists = fileExists(filePath);
  if (exists) {
    const backupFilePath = getBackupFilePath(filePath);
    fs.renameSync(filePath, backupFilePath);
  }

  return exists;
}

function getBackupFilePath(filePath: string): string {
  const parsedPath = path.parse(filePath);
  parsedPath.base = parsedPath.name + '.old' + parsedPath.ext;
  return path.normalize(path.format(parsedPath));
}

function fileExists(filePath: string): boolean {
  try {
    fs.accessSync(filePath);
    return true;
  } catch {
    return false;
  }
}

// When cleaning up old log files they are first backed up and the next time removed.
export function cleanUpLogDirectory(): void {
  const oldLogFileNames = ['frontend-renderer.log'];

  oldLogFileNames.forEach((fileName) => {
    const filePath = path.join(getLogsDirectory(), fileName);
    rotateOrDeleteFile(filePath);
  });
}

export function rotateOrDeleteFile(filePath: string) {
  const backupFilePath = getBackupFilePath(filePath);
  if (!backupLogFile(filePath) && fileExists(backupFilePath)) {
    fs.unlinkSync(backupFilePath);
  }
}
