import { IpcRendererEventChannel } from '../../shared/ipc-event-channel';
import { LogOutput } from '../../shared/logging';
import { LogLevel } from '../../shared/logging-types';

export default class IpcOutput extends LogOutput {
  protected writeImpl(level: LogLevel, message: string) {
    IpcRendererEventChannel.logging.log({ level: level, message });
  }
}
