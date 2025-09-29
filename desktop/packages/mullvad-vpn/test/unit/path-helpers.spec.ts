import { expect } from 'chai';
import { describe, it } from 'mocha';

import { matchPaths } from '../e2e/lib/path-helpers';

describe('E2E test path helper', () => {
  it('should identify matching paths', () => {
    expect(matchPaths('/a/b/c', '/a/b/c')).to.be.true;
    expect(matchPaths('/a/b/c', '/a/b/c/')).to.be.true;
    expect(matchPaths('/a/b/:param', '/a/b/c')).to.be.true;
    expect(matchPaths('/a/:param/:param', '/a/b/c')).to.be.true;
    expect(matchPaths('/a/:param/:param?', '/a/b/c')).to.be.true;
    expect(matchPaths('/a/:param/:param?', '/a/b')).to.be.true;
    expect(matchPaths('/a/:param?/:param?', '/a')).to.be.true;

    expect(matchPaths('/a/b/c', '/a/b')).to.be.false;
    expect(matchPaths('/a/b/c', '/a/b/d')).to.be.false;
    expect(matchPaths('/a/b/c', '/a/b/c/d')).to.be.false;
    expect(matchPaths('/a/b/c', 'a/b/c')).to.be.false;
    expect(matchPaths('/a/b/:param', '/a/b')).to.be.false;

    expect(() => matchPaths('/a/b/c', '/a/b/:param')).to.throw();
  });
});
