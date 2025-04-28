import { IpcRendererEventChannel } from '../../src/renderer/lib/ipc-event-channel';

// The ViewTransition types can be removed from here whenever TS adds support for them.
interface ViewTransition {
  readonly ready: Promise<void>;
  readonly finished: Promise<void>;
}

declare global {
  interface Window {
    ipc: typeof IpcRendererEventChannel;
    env: { platform: NodeJS.Platform; development: boolean; e2e: boolean };
    e2e: { location: string };
  }

  // The ViewTransition types can be removed from here whenever TS adds support for them.
  interface Document {
    startViewTransition(callback: () => void): ViewTransition;
  }
}
