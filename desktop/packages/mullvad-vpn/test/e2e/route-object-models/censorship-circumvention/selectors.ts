import { Page } from 'playwright';

export const createSelectors = (page: Page) => ({
  udpOverTcpSettingsButton: () => page.getByRole('button', { name: 'UDP-over-TCP settings' }),
  udpOverTcpOption: () => page.getByRole('option', { name: 'UDP-over-TCP' }),
  automaticOption: () =>
    page
      .getByRole('listbox', { name: 'Method' })
      .getByRole('option', { name: 'Automatic', exact: true }),
  shadowsocksSettingsButton: () => page.getByRole('button', { name: 'Shadowsocks settings' }),
  shadowsocksOption: () => page.getByRole('option', { name: 'Shadowsocks' }),
  lwoOption: () => page.getByRole('option', { name: 'LWO' }),
  quicOption: () => page.getByRole('option', { name: 'QUIC' }),
  portObfuscationOption: () => page.getByRole('option', { name: 'Port', exact: true }),
  wireguardPortOption: () => page.getByRole('option', { name: 'WireGuard Port' }),
});
