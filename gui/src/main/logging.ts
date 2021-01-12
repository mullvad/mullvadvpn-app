import { app } from 'electron';
import fs from 'fs';
import path from 'path';
import { IpcMainEventChannel } from '../shared/ipc-event-channel';
import { LogOutput } from '../shared/logging';
import { LogLevel, ILogInput, LogLevels } from '../shared/logging-types';

export const OLD_LOG_FILES = ['frontend-renderer.log'];

export class FileOutput extends LogOutput {
  private fileDescriptor: number;

  constructor(level: LogLevel, private filePath: string) {
    super(level);

    this.fileDescriptor = fs.openSync(filePath, fs.constants.O_CREAT | fs.constants.O_WRONLY);
  }

  public dispose() {
    fs.closeSync(this.fileDescriptor);
  }

  protected writeImpl(_level: LogLevel, message: string) {
    fs.write(this.fileDescriptor, `${message}\n`, (err) => {
      if (err) {
        console.error(`Failed to log to ${this.filePath}`);
      }
    });
  }
}

export class IpcInput implements ILogInput {
  public on(handler: (level: LogLevel, message: string) => void) {
    IpcMainEventChannel.logging.handleLog(({ level, message }) =>
      handler(LogLevels[level], message),
    );
  }
}

export function getMainLogPath() {
  return path.join(getLogDirectoryDir(), 'main.log');
}

export function getRendererLogPath() {
  return path.join(getLogDirectoryDir(), 'renderer.log');
}

export function createLoggingDirectory(): void {
  fs.mkdirSync(getLogDirectoryDir(), { recursive: true });
}

// When cleaning up old log files they are first backed up and the next time removed.
export function cleanUpLogDirectory(fileNames: string[]): void {
  fileNames.forEach((fileName) => {
    const filePath = path.join(getLogDirectoryDir(), fileName);
    rotateOrDeleteFile(filePath);
  });
}

export function backupLogFile(filePath: string): boolean {
  const exists = fileExists(filePath);
  if (exists) {
    const backupFilePath = getBackupFilePath(filePath);
    fs.renameSync(filePath, backupFilePath);
  }

  return exists;
}

export function rotateOrDeleteFile(filePath: string): void {
  const backupFilePath = getBackupFilePath(filePath);
  if (!backupLogFile(filePath) && fileExists(backupFilePath)) {
    fs.unlinkSync(backupFilePath);
  }
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

function getLogDirectoryDir() {
  return app.getPath('logs');
}
