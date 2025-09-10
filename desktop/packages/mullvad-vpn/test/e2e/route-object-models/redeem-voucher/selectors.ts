import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  voucherInput: () => page.getByPlaceholder('XXXX-XXXX-XXXX-XXXX'),
  redeemButton: () => page.getByRole('button', { name: 'Redeem' }),
});
