import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  redeemVoucherButton: () => page.getByRole('button', { name: 'Redeem voucher' }),
});
