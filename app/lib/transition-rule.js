// @flow

export type TransitionDescriptor = {
  name: string,
  duration: number
};

export type TransitionFork = {
  forward: TransitionDescriptor,
  backward: TransitionDescriptor
};

/**
 * Transition rule
 *
 * @class TransitionRule
 */
export default class TransitionRule {

  from: ?string;
  to: string;
  fork: TransitionFork;
  dir: 'forward' | 'backward' = 'forward';

  /**
   * Creates an instance of TransitionRule.
   * @param {string} from - source route to match against, pass null for any.
   * @param {string} to - destination route to match against
   * @param {TransitionFork} fork - transition
   *
   * @memberof TransitionRule
   */
  constructor(from: ?string, to: string, fork: TransitionFork) {
    this.from = from;
    this.to = to;
    this.fork = fork;
  }

  /**
   * Attempts to match the transition between routes A -> B and B -> A
   *
   * @param {string} [fromRoute] source route, pass null for any
   * @param {string} toRoute
   * @returns {boolean} true if matches, otherwise false
   *
   * @memberof TransitionRule
   */
  match(fromRoute: ?string, toRoute: string): boolean {
    if((!this.from || this.from === fromRoute) && this.to === toRoute) {
      this.dir = 'forward';
      return true;
    }

    if((!this.from || this.from === toRoute) && this.to === fromRoute) {
      this.dir = 'backward';
      return true;
    }

    return false;
  }

  /**
   * Returns transition descriptor.
   * Make sure you run match() before to obtain the direction
   * of transition before calling this method
   *
   * @returns {TransitionDescriptor} transitionDescriptor
   *
   * @memberof TransitionRule
   */
  transitionDescriptor(): TransitionDescriptor {
    return this.fork[this.dir];
  }
}