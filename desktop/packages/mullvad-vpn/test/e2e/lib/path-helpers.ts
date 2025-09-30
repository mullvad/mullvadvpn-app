import { expect } from '@playwright/test';

import { RoutePath } from '../../../src/shared/routes';

export function generatePath(path: RoutePath, params: Record<string, string>): string {
  return Object.entries(params)
    .reduce(
      (path, [name, value]) => path.replace(new RegExp(`:${name}\\??`), value),
      path as string,
    )
    .replaceAll(new RegExp('/:.*?\\?', 'g'), '');
}

// Match the actual path against against the expected path where the expected can contain parameters
function toMatchPath(actual: string, expected: string | null) {
  const pass = matchPaths(expected, actual);
  const message = () =>
    pass
      ? `Expected path to be "${expected}"`
      : `Expected path "${expected}", but found "${actual}"`;
  return { pass, message };
}

expect.extend({ toMatchPath });

function trimTrailingSlash(value: string): string {
  return value.replaceAll(/\/$/g, '');
}

// Match b against a where a can contain parameters
export function matchPaths(a: string | null, b: string | null): boolean {
  if (a === null || b === null) {
    return a === b;
  }

  const aParts = trimTrailingSlash(a).split('/');
  const bParts = trimTrailingSlash(b).split('/');

  if (bParts.some((part) => part.startsWith(':'))) {
    throw new Error('Only first argument is allowed to contain dynamic route path segments');
  }

  return (
    aParts.length >= bParts.length &&
    aParts.every((aPart, i) => {
      if (aPart.startsWith(':')) {
        return aPart.endsWith('?') ? true : bParts[i] !== undefined;
      } else {
        return aPart === bParts[i];
      }
    })
  );
}
