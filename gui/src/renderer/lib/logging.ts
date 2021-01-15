import { IpcRendererEventChannel } from '../../shared/ipc-event-channel';
import { ILogOutput, LogLevel } from '../../shared/logging-types';

export default class IpcOutput implements ILogOutput {
  constructor(public level: LogLevel) {}

  public write(level: LogLevel, message: string) {
    IpcRendererEventChannel.logging.log({ level: level, message });
  }
}
