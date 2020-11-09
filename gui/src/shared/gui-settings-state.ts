// This is a special value which is when contained within IGuiSettingsState.preferredLocale
// indicates that app should use the active operating system locale to determine the UI language.
export const SYSTEM_PREFERRED_LOCALE_KEY = 'system';

export interface IGuiSettingsState {
  // A user interface locale.
  // Use 'system' to opt-in for active locale set in the operating system
  // (see SYSTEM_PREFERRED_LOCALE_KEY)
  preferredLocale: string;

  // Enable or disable system notifications on tunnel state etc.
  enableSystemNotifications: boolean;

  // Tells the app to activate auto-connect feature in the mullvad-daemon, but only if the app is
  // set to auto-start with the system.
  autoConnect: boolean;

  // Tells the app to use monochromatic set of icons for tray.
  monochromaticIcon: boolean;

  // Tells the app to hide the main window on start.
  startMinimized: boolean;

  // Tells the app wheter or not it should act as a window or a context menu.
  unpinnedWindow: boolean;
}
