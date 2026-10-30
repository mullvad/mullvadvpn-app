import { IpcRendererEventChannel } from '../../src/renderer/lib/ipc-event-channel';

declare global {
  interface Window {
    ipc: typeof IpcRendererEventChannel;
    env: { platform: NodeJS.Platform; development: boolean; e2e: boolean };
    e2e: { location: string };
  }
  // The activeViewTransition type can be removed from here whenever TS adds support for them.
  interface Document {
    readonly activeViewTransition: ViewTransition | null;
  }
}
