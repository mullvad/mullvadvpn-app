import { IpcRendererEventChannel } from '../../src/renderer/lib/ipc-event-channel';

declare global {
  interface Window {
    ipc: typeof IpcRendererEventChannel;
    platform: NodeJS.Platform;
    runningInDevelopment: boolean;
  }
}
