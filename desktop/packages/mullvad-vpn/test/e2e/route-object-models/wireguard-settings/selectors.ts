import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  udpOverTcpSettingsButton: () => page.getByRole('button', { name: 'UDP-over-TCP settings' }),
  udpOverTcpOption: () => page.getByRole('option', { name: 'UDP-over-TCP' }),
  automaticObfuscationOption: () =>
    page
      .getByRole('listbox', { name: 'Obfuscation' })
      .getByRole('option', { name: 'Automatic', exact: true }),
});
