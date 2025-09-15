import { generatePath } from 'react-router';

import { RoutePath } from '../../shared/routes';

export type GeneratedRoutePath = { routePath: string };

export function generateRoutePath(
  routePath: RoutePath,
  parameters: Parameters<typeof generatePath>[1],
): GeneratedRoutePath {
  return { routePath: generatePath(routePath, parameters) };
}
