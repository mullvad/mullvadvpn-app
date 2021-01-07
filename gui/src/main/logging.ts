import { app } from 'electron';
import fs from 'fs';
import path from 'path';
import { IpcMainEventChannel } from './ipc-event-channel';
import { LogLevel, ILogInput, ILogOutput } from '../shared/logging-types';

export const OLD_LOG_FILES = ['frontend-renderer.log'];

export class FileOutput implements ILogOutput {
  private fileDescriptor?: number;

  constructor(public level: LogLevel, private filePath: string) {
    try {
      this.fileDescriptor = fs.openSync(filePath, fs.constants.O_CREAT | fs.constants.O_WRONLY);
    } catch (e) {
      console.error(`Failed to open ${this.filePath}`);
    }
  }

  public dispose() {
    if (this.fileDescriptor) {
      try {
        fs.closeSync(this.fileDescriptor);
      } catch (e) {
        console.error(`Failed to close ${this.filePath}`);
      }
    }
  }

  public write(_level: LogLevel, message: string) {
    if (this.fileDescriptor) {
      fs.write(this.fileDescriptor, `${message}\n`, (err) => {
        if (err) {
          console.error(`Failed to log to ${this.filePath}`);
        }
      });
    }
  }
}

export class IpcInput implements ILogInput {
  public on(handler: (level: LogLevel, message: string) => void) {
    IpcMainEventChannel.logging.handleLog(({ level, message }) => handler(level, message));
  }
}

export function getMainLogPath() {
  return path.join(getLogDirectoryDir(), 'main.log');
}

export function getRendererLogPath() {
  return path.join(getLogDirectoryDir(), 'renderer.log');
}

export function createLoggingDirectory(): void {
  try {
    fs.mkdirSync(getLogDirectoryDir(), { recursive: true });
  } catch (e) {
    console.error('Failed to create logging directory');
  }
}

// When cleaning up old log files they are first backed up and the next time removed.
export function cleanUpLogDirectory(fileNames: string[]): void {
  fileNames.forEach((fileName) => {
    const filePath = path.join(getLogDirectoryDir(), fileName);
    rotateOrDeleteFile(filePath);
  });
}

export function backupLogFile(filePath: string): boolean {
  const backupFilePath = getBackupFilePath(filePath);
  try {
    fs.accessSync(filePath);
    fs.renameSync(filePath, backupFilePath);
    return true;
  } catch (e) {
    console.error(`Failed to backup ${filePath}`);
    return false;
  }
}

export function rotateOrDeleteFile(filePath: string): void {
  if (!backupLogFile(filePath)) {
    const backupFilePath = getBackupFilePath(filePath);
    try {
      fs.accessSync(backupFilePath);
      fs.unlinkSync(backupFilePath);
    } catch (e) {
      console.error(`Failed to delete ${filePath}`);
    }
  }
}

function getBackupFilePath(filePath: string): string {
  const parsedPath = path.parse(filePath);
  parsedPath.base = parsedPath.name + '.old' + parsedPath.ext;
  return path.normalize(path.format(parsedPath));
}

function getLogDirectoryDir() {
  return app.getPath('logs');
}
