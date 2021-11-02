import { generatePath } from 'react-router';

export type GeneratedRoutePath = { routePath: string };

export enum RoutePath {
  launch = '/',
  login = '/login',
  main = '/main',
  redeemVoucher = '/main/voucher/redeem',
  voucherSuccess = '/main/voucher/success/:newExpiry/:secondsAdded',
  timeAdded = '/main/time-added',
  setupFinished = '/main/setup-finished',
  settings = '/settings',
  selectLanguage = '/settings/language',
  accountSettings = '/settings/account',
  preferences = '/settings/preferences',
  advancedSettings = '/settings/advanced',
  wireguardSettings = '/settings/advanced/wireguard',
  openVpnSettings = '/settings/advanced/openvpn',
  splitTunneling = '/settings/advanced/split-tunneling',
  support = '/settings/support',
  selectLocation = '/select-location',
  filterByProvider = '/select-location/filter-by-provider',
}

export function generateRoutePath(
  routePath: RoutePath,
  parameters: Parameters<typeof generatePath>[1],
): GeneratedRoutePath {
  return { routePath: generatePath(routePath, parameters) };
}
