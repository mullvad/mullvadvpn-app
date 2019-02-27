import { expect } from 'chai';
import { it, describe } from 'mocha';
import TransitionRule from '../src/renderer/lib/transition-rule';

describe('TransitionRule', () => {
  const testTransition = {
    forward: { name: 'forward', duration: 0.25 },
    backward: { name: 'backward', duration: 0.25 },
  };

  it('should match wildcard rule', () => {
    const rule = new TransitionRule(null, '/route', testTransition);

    expect(rule.match(null, '/route')).to.deep.equal({
      direction: 'forward',
      descriptor: { name: 'forward', duration: 0.25 },
    });

    expect(rule.match('/somewhere', '/route')).to.deep.equal({
      direction: 'forward',
      descriptor: { name: 'forward', duration: 0.25 },
    });
  });

  it('should match wildcard rule reversion', () => {
    const rule = new TransitionRule(null, '/route', testTransition);

    expect(rule.match('/route', '/other')).to.deep.equal({
      direction: 'backward',
      descriptor: { name: 'backward', duration: 0.25 },
    });
  });

  it('should match exact rule', () => {
    const rule = new TransitionRule('/route1', '/route2', testTransition);

    expect(rule.match('/other', '/route1')).to.be.null;
    expect(rule.match('/other', '/route2')).to.be.null;

    expect(rule.match('/route1', '/route2')).to.deep.equal({
      direction: 'forward',
      descriptor: { name: 'forward', duration: 0.25 },
    });
  });

  it('should match exact rule reversion', () => {
    const rule = new TransitionRule('/route1', '/route2', testTransition);

    expect(rule.match('/route1', '/other')).to.be.null;
    expect(rule.match('/route2', '/other')).to.be.null;

    expect(rule.match('/route2', '/route1')).to.deep.equal({
      direction: 'backward',
      descriptor: { name: 'backward', duration: 0.25 },
    });
  });
});
