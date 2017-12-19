// @flow

import TransitionRule from './lib/transition-rule';
import type { TransitionFork, TransitionDescriptor } from './lib/transition-rule';

export type CSSTransitionGroupProps = {
  transitionName: string,
  transitionEnterTimeout: number,
  transitionLeaveTimeout: number,
  transitionEnter: boolean,
  transitionLeave: boolean,
  transitionAppear?: boolean,
  transitionAppearTimeout?: number
};

type TransitionMap = {
  [name: string]: TransitionFork
};

/**
 * Calculate CSSTransitionGroup props.
 *
 * @param {string} [fromRoute] - source route
 * @param {string} toRoute     - target route
 */
export const getTransitionProps = (fromRoute: ?string, toRoute: string): CSSTransitionGroupProps => {
  // ignore initial transition and transition between the same routes
  if(!fromRoute || fromRoute === toRoute) {
    return noTransitionProps();
  }

  for(const rule of transitionRules) {
    const match = rule.match(fromRoute, toRoute);
    if(match) {
      return toCSSTransitionGroupProps(match.descriptor);
    }
  }

  return noTransitionProps();
};

/**
 * Integrate TransitionDescriptor into CSSTransitionGroupProps
 * @param {TransitionDescriptor} descriptor
 */
const toCSSTransitionGroupProps = (descriptor: TransitionDescriptor): CSSTransitionGroupProps => {
  const {name, duration} = descriptor;
  return {
    transitionName: name,
    transitionEnterTimeout: duration,
    transitionLeaveTimeout: duration,
    transitionEnter: true,
    transitionLeave: true
  };
};

/**
 * Returns default props with animations disabled
 */
const noTransitionProps = (): CSSTransitionGroupProps => ({
  transitionName: '',
  transitionEnterTimeout: 0,
  transitionLeaveTimeout: 0,
  transitionEnter: false,
  transitionLeave: false
});

/**
 * Transition descriptors
 */
const transitions: TransitionMap = {
  slide: {
    forward: {
      name: 'slide-up-transition',
      duration: 450
    },
    backward: {
      name: 'slide-down-transition',
      duration: 450
    }
  },
  push: {
    forward: {
      name: 'push-transition',
      duration: 450
    },
    backward: {
      name: 'pop-transition',
      duration: 450
    }
  }
};

/**
 * Shortcut to create TransitionRule
 */
const r = (from: ?string, to: string, fork: TransitionFork): TransitionRule => {
  return new TransitionRule(from, to, fork);
};

/**
 * Transition rules
 * (null) is used to indicate any route.
 */
const transitionRules = [
  r('/settings', '/settings/account', transitions.push),
  r(null, '/settings', transitions.slide),
  r(null, '/select-location', transitions.slide)
];