import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  continueButton: () => page.getByRole('button', { name: 'Continue' }),
  removeDeviceButtons: () => page.getByRole('button', { name: 'Remove device named' }),
  confirmRemoveDeviceButton: () => page.getByRole('button', { name: 'Remove', exact: true }),
});
