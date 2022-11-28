import { generatePath } from 'react-router';

export type GeneratedRoutePath = { routePath: string };

export enum RoutePath {
  launch = '/',
  login = '/login',
  tooManyDevices = '/login/too-many-devices',
  deviceRevoked = '/login/device-revoked',
  main = '/main',
  redeemVoucher = '/main/voucher/redeem',
  voucherSuccess = '/main/voucher/success/:newExpiry/:secondsAdded',
  timeAdded = '/main/time-added',
  setupFinished = '/main/setup-finished',
  settings = '/settings',
  selectLanguage = '/settings/language',
  accountSettings = '/settings/account',
  userInterfaceSettings = '/settings/interface',
  vpnSettings = '/settings/vpn',
  wireguardSettings = '/settings/advanced/wireguard',
  openVpnSettings = '/settings/advanced/openvpn',
  splitTunneling = '/settings/split-tunneling',
  support = '/settings/support',
  problemReport = '/settings/support/problem-report',
  debug = '/settings/debug',
  selectLocation = '/select-location',
  filter = '/select-location/filter',
}

export const disableDismissForRoutes = [
  RoutePath.launch,
  RoutePath.login,
  RoutePath.tooManyDevices,
  RoutePath.deviceRevoked,
  RoutePath.main,
  RoutePath.redeemVoucher,
  RoutePath.voucherSuccess,
  RoutePath.timeAdded,
  RoutePath.setupFinished,
];

export function generateRoutePath(
  routePath: RoutePath,
  parameters: Parameters<typeof generatePath>[1],
): GeneratedRoutePath {
  return { routePath: generatePath(routePath, parameters) };
}
