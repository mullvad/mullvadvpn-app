export interface ITransitionDescriptor {
  name: string;
  duration: number;
}

export interface ITransitionFork {
  forward: ITransitionDescriptor;
  backward: ITransitionDescriptor;
}

export interface ITransitionMatch {
  direction: 'forward' | 'backward';
  descriptor: ITransitionDescriptor;
}

export default class TransitionRule {
  private from: RegExp;
  private to: RegExp;

  constructor(from: string | RegExp, to: string | RegExp, private fork: ITransitionFork) {
    this.from = typeof from === 'string' ? new RegExp(from) : from;
    this.to = typeof to === 'string' ? new RegExp(to) : to;
  }

  public match(fromRoute: string, toRoute: string): ITransitionMatch | null {
    if (this.from.test(fromRoute) && this.to.test(toRoute)) {
      return {
        direction: 'forward',
        descriptor: this.fork.forward,
      };
    }

    if (this.from.test(toRoute) && this.to.test(fromRoute)) {
      return {
        direction: 'backward',
        descriptor: this.fork.backward,
      };
    }

    return null;
  }
}
