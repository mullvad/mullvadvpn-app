// @flow

import TransitionRule from './lib/transition-rule';
import type { TransitionFork, TransitionDescriptor } from './lib/transition-rule';

export type TransitionGroupProps = {
  name: string,
  duration: number,
};

type TransitionMap = {
  [name: string]: TransitionFork,
};

/**
 * Transition descriptors
 */
const transitions: TransitionMap = {
  slide: {
    forward: {
      name: 'slide-up',
      duration: 450,
    },
    backward: {
      name: 'slide-down',
      duration: 450,
    },
  },
  push: {
    forward: {
      name: 'push',
      duration: 450,
    },
    backward: {
      name: 'pop',
      duration: 450,
    },
  },
};

/**
 * Transition rules
 * (null) is used to indicate any route.
 */
const transitionRules = [
  r('/settings', '/settings/account', transitions.push),
  r('/settings', '/settings/preferences', transitions.push),
  r('/settings', '/settings/advanced', transitions.push),
  r('/settings', '/settings/support', transitions.push),
  r(null, '/settings', transitions.slide),
  r(null, '/select-location', transitions.slide),
];

/**
 * Calculate TransitionGroup props.
 *
 * @param {string} [fromRoute] - source route
 * @param {string} toRoute     - target route
 */
export function getTransitionProps(fromRoute: ?string, toRoute: string): TransitionGroupProps {
  // ignore initial transition and transition between the same routes
  if (!fromRoute || fromRoute === toRoute) {
    return noTransitionProps();
  }

  for (const rule of transitionRules) {
    const match = rule.match(fromRoute, toRoute);
    if (match) {
      return toTransitionGroupProps(match.descriptor);
    }
  }

  return noTransitionProps();
}

/**
 * Integrate TransitionDescriptor into TransitionGroupProps
 * @param {TransitionDescriptor} descriptor
 */
function toTransitionGroupProps(descriptor: TransitionDescriptor): TransitionGroupProps {
  const { name, duration } = descriptor;
  return {
    name: name,
    duration: duration,
  };
}

/**
 * Returns default props with no animation
 */
function noTransitionProps(): TransitionGroupProps {
  return {
    name: '',
    duration: 0,
  };
}

/**
 * Shortcut to create TransitionRule
 */
function r(from: ?string, to: string, fork: TransitionFork): TransitionRule {
  return new TransitionRule(from, to, fork);
}
