import TransitionRule, { ITransitionDescriptor, ITransitionFork } from './lib/transition-rule';

export interface ITransitionGroupProps {
  name: string;
  duration: number;
}

interface ITransitionMap {
  [name: string]: ITransitionFork;
}

/**
 * Transition descriptors
 */
const transitions: ITransitionMap = {
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
  r('/settings', '/settings/language', transitions.push),
  r('/settings', '/settings/account', transitions.push),
  r('/settings', '/settings/preferences', transitions.push),
  r('/settings', '/settings/advanced', transitions.push),
  r('/settings/advanced', '/settings/advanced/wireguard-keys', transitions.push),
  r('/settings/advanced', '/settings/advanced/linux-split-tunneling', transitions.push),
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
export function getTransitionProps(
  fromRoute: string | null,
  toRoute: string,
): ITransitionGroupProps {
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
 * Integrate ITransitionDescriptor into ITransitionGroupProps
 * @param {ITransitionDescriptor} descriptor
 */
function toTransitionGroupProps(descriptor: ITransitionDescriptor): ITransitionGroupProps {
  const { name, duration } = descriptor;
  return {
    name,
    duration,
  };
}

/**
 * Returns default props with no animation
 */
function noTransitionProps(): ITransitionGroupProps {
  return {
    name: '',
    duration: 0,
  };
}

/**
 * Shortcut to create TransitionRule
 */
function r(from: string | null, to: string, fork: ITransitionFork): TransitionRule {
  return new TransitionRule(from, to, fork);
}
