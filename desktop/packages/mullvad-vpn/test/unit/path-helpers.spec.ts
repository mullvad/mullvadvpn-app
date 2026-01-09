import { describe, expect, it } from 'vitest';

import { RoutePath } from '../../src/shared/routes';
import { generatePath, matchPaths } from '../e2e/lib/path-helpers';

describe('E2E test path helper', () => {
  it('should identify matching paths', () => {
    expect(matchPaths('/a/b/c', '/a/b/c')).toBe(true);
    expect(matchPaths('/a/b/c', '/a/b/c/')).toBe(true);
    expect(matchPaths('/a/b/:param', '/a/b/c')).toBe(true);
    expect(matchPaths('/a/:param/:param', '/a/b/c')).toBe(true);
    expect(matchPaths('/a/:param/:param?', '/a/b/c')).toBe(true);
    expect(matchPaths('/a/:param/:param?', '/a/b')).toBe(true);
    expect(matchPaths('/a/:param?/:param?', '/a')).toBe(true);

    expect(matchPaths('/a/b/c', '/a/b')).toBe(false);
    expect(matchPaths('/a/b/c', '/a/b/d')).toBe(false);
    expect(matchPaths('/a/b/c', '/a/b/c/d')).toBe(false);
    expect(matchPaths('/a/b/c', 'a/b/c')).toBe(false);
    expect(matchPaths('/a/b/:param', '/a/b')).toBe(false);

    expect(() => matchPaths('/a/b/c', '/a/b/:clock')).toThrow();
    expect(() => matchPaths('/a/b/:clock', '/a/b/20:00')).not.toThrow();
  });

  it('should correctly replace parameters', () => {
    expect(generatePath('/a/b' as RoutePath, {})).to.equal('/a/b');
    expect(generatePath('/a/:param' as RoutePath, { param: 'b' })).to.equal('/a/b');
    expect(generatePath('/a/:param?' as RoutePath, { param: 'b' })).to.equal('/a/b');
    expect(generatePath('/a/:param?' as RoutePath, {})).to.equal('/a');
  });
});
