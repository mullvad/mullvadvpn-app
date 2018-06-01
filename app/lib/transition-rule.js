// @flow

export type TransitionDescriptor = {
  name: string,
  duration: number,
};

export type TransitionFork = {
  forward: TransitionDescriptor,
  backward: TransitionDescriptor,
};

export type TransitionMatch = {
  direction: 'forward' | 'backward',
  descriptor: TransitionDescriptor,
};

export default class TransitionRule {
  _from: ?string;
  _to: string;
  _fork: TransitionFork;

  constructor(from: ?string, to: string, fork: TransitionFork) {
    this._from = from;
    this._to = to;
    this._fork = fork;
  }

  match(fromRoute: ?string, toRoute: string): ?TransitionMatch {
    if ((!this._from || this._from === fromRoute) && this._to === toRoute) {
      return {
        direction: 'forward',
        descriptor: this._fork['forward'],
      };
    }

    if ((!this._from || this._from === toRoute) && this._to === fromRoute) {
      return {
        direction: 'backward',
        descriptor: this._fork['backward'],
      };
    }

    return null;
  }
}
