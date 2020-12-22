import Gettext from 'node-gettext';
import { IpcRendererEventChannel } from '../../src/shared/ipc-event-channel';

declare global {
  interface Window {
    loadTranslations(locale: string, catalogue: Gettext): void;
    ipc: typeof IpcRendererEventChannel;
  }
}
