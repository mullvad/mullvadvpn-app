import { IpcRendererEventChannel } from '../../src/shared/ipc-event-channel';

declare global {
  interface Window {
    ipc: typeof IpcRendererEventChannel;
  }
}
