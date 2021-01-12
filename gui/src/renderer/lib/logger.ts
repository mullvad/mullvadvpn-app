import moment from 'moment';
import { IpcRendererEventChannel } from '../../shared/ipc-event-channel';
import { Logger, LogLevel } from '../../shared/logging-types';

export default class RendererLogger implements Logger {
  private consoleLevel?: LogLevel;
  private ipcLevel?: LogLevel;

  public init(consoleLevel?: LogLevel, ipcLevel?: LogLevel) {
    this.consoleLevel = consoleLevel;
    this.ipcLevel = ipcLevel;
  }

  public log(level: LogLevel, ...data: unknown[]): void {
    const time = moment().format('HH:mm:ss.SSSS');
    const message = `[${time}][${level.name}] ${data.join(' ')}`;

    if (this.consoleLevel && this.consoleLevel.level >= level.level) {
      level.consoleFunction(message);
    }

    if (this.ipcLevel && this.ipcLevel.level >= level.level) {
      IpcRendererEventChannel.logging.log({ level: level.name, data });
    }
  }
}
