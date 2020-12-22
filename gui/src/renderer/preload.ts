import { loadTranslations } from '../shared/gettext';
import { IpcRendererEventChannel } from '../shared/ipc-event-channel';

window.loadTranslations = (locale, catalogue) => loadTranslations(locale, catalogue);
window.ipc = IpcRendererEventChannel;
