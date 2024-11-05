import { ILogOutput, LogLevel } from '../../shared/logging-types';

export default class IpcOutput implements ILogOutput {
  constructor(public level: LogLevel) {}

  public write(level: LogLevel, message: string) {
    window.ipc.logging.log({ level: level, message });
  }
}
