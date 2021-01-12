import { app } from 'electron';
import fs from 'fs';
import moment from 'moment';
import path from 'path';
import { Logger, LogLevel, LogLevels } from '../shared/logging-types';

export default class MainLogger implements Logger {
  private file?: string;
  private consoleLevel?: LogLevel;
  private fileLevel?: LogLevel;

  private fileDescriptor?: number;

  public init(fileName?: string, consoleLevel?: LogLevel, fileLevel?: LogLevel): void {
    this.file = fileName ? path.join(MainLogger.getLogDirectoryDir(), fileName) : undefined;
    this.consoleLevel = consoleLevel;
    this.fileLevel = fileLevel;

    if (process.env.NODE_ENV !== 'development') {
      // Ensure log directory exists
      fs.mkdirSync(MainLogger.getLogDirectoryDir(), { recursive: true });

      if (this.file) {
        MainLogger.backupLogFile(this.file);
      }

      // Don't log to file during development
      this.openFile();
    }

    this.log(LogLevels.debug, `Logging to ${this.file}`);
  }

  // When cleaning up old log files they are first backed up and the next time removed.
  public static cleanUpLogDirectory(fileNames: string[]): void {
    fileNames.forEach((fileName) => {
      const filePath = path.join(MainLogger.getLogDirectoryDir(), fileName);
      MainLogger.rotateOrDeleteFile(filePath);
    });
  }

  public log(level: LogLevel, ...data: unknown[]): void {
    const time = moment().format('YYYY-MM-DD HH:mm:ss.SSS');
    const message = `[${time}][${level.name}] ${data.join(' ')}`;

    if (this.consoleLevel && this.consoleLevel.level >= level.level) {
      level.consoleFunction(message);
    }

    if (this.fileDescriptor && this.fileLevel && this.fileLevel.level >= level.level) {
      try {
        fs.appendFileSync(this.fileDescriptor, message + '\n');
      } catch (e) {
        // Use console.error since we don't want to call logger and get stuck in infinite recursion.
        console.error(`Failed to log to ${this.file}`);
      }
    }
  }

  public static backupLogFile(filePath: string): boolean {
    const exists = MainLogger.fileExists(filePath);
    if (exists) {
      const backupFilePath = MainLogger.getBackupFilePath(filePath);
      fs.renameSync(filePath, backupFilePath);
    }

    return exists;
  }

  public static rotateOrDeleteFile(filePath: string): void {
    const backupFilePath = MainLogger.getBackupFilePath(filePath);
    if (!MainLogger.backupLogFile(filePath) && MainLogger.fileExists(backupFilePath)) {
      fs.unlinkSync(backupFilePath);
    }
  }

  public closeFile() {
    if (this.fileDescriptor) {
      fs.closeSync(this.fileDescriptor);
    }
  }

  private openFile() {
    if (this.file) {
      this.fileDescriptor = fs.openSync(this.file, fs.constants.O_CREAT | fs.constants.O_WRONLY);
    }
  }

  private static getBackupFilePath(filePath: string): string {
    const parsedPath = path.parse(filePath);
    parsedPath.base = parsedPath.name + '.old' + parsedPath.ext;
    return path.normalize(path.format(parsedPath));
  }

  private static fileExists(filePath: string): boolean {
    try {
      fs.accessSync(filePath);
      return true;
    } catch {
      return false;
    }
  }

  private static getLogDirectoryDir() {
    return app.getPath('logs');
  }
}
