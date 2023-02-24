import { generatePath } from 'react-router';

import { RoutePath } from './routes';

export type GeneratedRoutePath = { routePath: string };

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
