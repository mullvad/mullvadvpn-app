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
  private from: string | null;
  private to: string;
  private fork: ITransitionFork;

  constructor(from: string | null, to: string, fork: ITransitionFork) {
    this.from = from;
    this.to = to;
    this.fork = fork;
  }

  public match(fromRoute: string | null, toRoute: string): ITransitionMatch | null {
    if ((!this.from || this.from === fromRoute) && this.to === toRoute) {
      return {
        direction: 'forward',
        descriptor: this.fork.forward,
      };
    }

    if ((!this.from || this.from === toRoute) && this.to === fromRoute) {
      return {
        direction: 'backward',
        descriptor: this.fork.backward,
      };
    }

    return null;
  }
}
