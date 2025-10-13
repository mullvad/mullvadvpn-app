import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  languageButton: () =>
    page.locator('button', {
      has: page.locator('img'),
    }),
  languageButtonLabel: (label: string) =>
    page.locator('button', {
      hasText: label,
    }),
  autoStartSwitch: () => page.getByRole('switch', { name: 'Notifications' }),
  monochromaticTrayIconSwitch: () => page.getByRole('switch', { name: 'Monochromatic tray icon' }),
  unpinnedWindowSwitch: () => page.getByRole('switch', { name: 'Unpin app from taskbar' }),
  startMinimizedSwitch: () => page.getByRole('switch', { name: 'Start minimized' }),
  animateMapSwitch: () => page.getByRole('switch', { name: 'Animate map' }),
});
