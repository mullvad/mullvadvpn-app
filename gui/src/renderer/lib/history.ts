import { Location, Action, LocationListener, LocationDescriptor } from 'history';

// It currently isn't possible to implement this correctly with support for a generic state. State
// can be added as a generic type (<S = unknown>) after this issue has been resolved:
// https://github.com/DefinitelyTyped/DefinitelyTyped/issues/49060
type S = unknown;
export default class History {
  private listeners: LocationListener<S>[] = [];
  private entries: Location<S>[];
  private index = 0;
  private lastAction: Action = 'POP';

  public constructor(location: string | Location<S>, state?: S) {
    this.entries = [this.createLocation(location, state)];
  }

  public get location(): Location<S> {
    return this.entries[this.index];
  }

  public get length(): number {
    return this.entries.length;
  }

  public get action(): Action {
    return this.lastAction;
  }

  public push = (nextLocation: LocationDescriptor<S>, nextState?: S) => {
    const location = this.createLocation(nextLocation, nextState);
    this.lastAction = 'PUSH';
    this.index += 1;
    this.entries.splice(this.index, this.entries.length - this.index, location);
    this.notify();
  };

  public replace = (nextLocation: LocationDescriptor<S>, nextState?: S) => {
    this.entries[this.index] = this.createLocation(nextLocation, nextState);
    this.lastAction = 'REPLACE';
    this.notify();
  };

  public go = (n: number) => {
    if (this.canGo(n)) {
      this.index += n;
      this.lastAction = 'POP';
      this.notify();
    }
  };

  public goBack = () => this.go(-1);
  public goForward = () => this.go(1);

  public reset = () => {
    this.lastAction = 'POP';
    this.index = 0;
    this.notify();
  };

  public resetWith = (nextLocation: LocationDescriptor<S>, nextState?: S) => {
    this.entries = [this.createLocation(nextLocation, nextState)];
    this.lastAction = 'REPLACE';
    this.index = 0;
    this.notify();
  };

  public canGo(n: number) {
    const nextIndex = this.index + n;
    return nextIndex >= 0 && nextIndex < this.entries.length;
  }

  public listen(callback: LocationListener<S>) {
    this.listeners.push(callback);
    return () => (this.listeners = this.listeners.filter((listener) => listener !== callback));
  }

  public block(): () => void {
    throw Error('Not implemented');
  }

  public createHref(): string {
    throw Error('Not implemented');
  }

  private notify() {
    this.listeners.forEach((listener) => listener(this.location, this.action));
  }

  private createLocation(location: LocationDescriptor<S>, state?: S): Location<S> {
    if (typeof location === 'object') {
      return {
        pathname: location.pathname ?? this.location.pathname,
        search: location.search ?? '',
        hash: location.hash ?? '',
        state: location.state,
        key: location.key ?? this.getRandomKey(),
      };
    } else {
      return {
        pathname: location,
        search: '',
        hash: '',
        state,
        key: this.getRandomKey(),
      };
    }
  }

  private getRandomKey() {
    return Math.random().toString(36).substr(8);
  }
}
