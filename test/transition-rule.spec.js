// @flow
import { expect } from 'chai';
import TransitionRule from '../app/lib/transition-rule';

describe('TransitionRule', () => {
  const testTransition = {
    forward: { name: 'forward', duration: 0.25 },
    backward: { name: 'backward', duration: 0.25 }
  };

  it('should match wildcard rule', () => {
    const rule = new TransitionRule(null, '/route', testTransition);

    expect(rule.match(null, '/route')).to.be.true;
    expect(rule.transitionDescriptor().name).to.be.equal('forward');

    expect(rule.match('/somewhere', '/route')).to.be.true;
    expect(rule.transitionDescriptor().name).to.be.equal('forward');
  });

  it('should match wildcard rule reversion', () => {
    const rule = new TransitionRule(null, '/route', testTransition);

    expect(rule.match('/route', '/other')).to.be.true;
    expect(rule.transitionDescriptor().name).to.be.equal('backward');
  });

  it('should match exact rule', () => {
    const rule = new TransitionRule('/route1', '/route2', testTransition);

    expect(rule.match('/other', '/route1')).to.be.false;
    expect(rule.match('/other', '/route2')).to.be.false;

    expect(rule.match('/route1', '/route2')).to.be.true;
    expect(rule.transitionDescriptor().name).to.be.equal('forward');
  });

  it('should match exact rule reversion', () => {
    const rule = new TransitionRule('/route1', '/route2', testTransition);

    expect(rule.match('/route1', '/other')).to.be.false;
    expect(rule.match('/route2', '/other')).to.be.false;

    expect(rule.match('/route2', '/route1')).to.be.true;
    expect(rule.transitionDescriptor().name).to.be.equal('backward');
  });

});