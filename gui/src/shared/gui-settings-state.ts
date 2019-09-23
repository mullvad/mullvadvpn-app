export const SYSTEM_PREFERRED_LOCALE_KEY = 'system';

export interface IGuiSettingsState {
  preferredLocale: string;
  enableSystemNotifications: boolean;
  autoConnect: boolean;
  monochromaticIcon: boolean;
  startMinimized: boolean;
}
