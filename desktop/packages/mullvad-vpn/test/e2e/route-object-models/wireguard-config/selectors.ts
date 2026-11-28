import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  publicKey: () => page.getByTestId('wireguard-public-key'),
  ipv4Address: () => page.getByTestId('wireguard-ipv4'),
  ipv6Address: () => page.getByTestId('wireguard-ipv6'),
  removeButton: () => page.getByRole('button', { name: 'Remove config' }),
});
