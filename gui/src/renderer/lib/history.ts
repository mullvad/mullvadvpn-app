import { Location, Action, LocationDescriptor } from 'history';

type LocationListener<S = unknown> = (
  location: Location<S>,
  action: Action,
  previousLocation: Location<S>,
) => void;

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
    const previousLocation = this.location;
    const location = this.createLocation(nextLocation, nextState);
    this.lastAction = 'PUSH';
    this.index += 1;
    this.entries.splice(this.index, this.entries.length - this.index, location);
    this.notify(previousLocation);
  };

  public replace = (nextLocation: LocationDescriptor<S>, nextState?: S) => {
    const previousLocation = this.location;
    this.entries[this.index] = this.createLocation(nextLocation, nextState);
    this.lastAction = 'REPLACE';
    this.notify(previousLocation);
  };

  public go = (n: number) => {
    if (this.canGo(n)) {
      const previousLocation = this.location;
      const nextIndex = this.index + n;

      this.index = nextIndex;
      this.lastAction = 'POP';
      this.notify(previousLocation);
    }
  };

  public goBack = () => this.go(-1);
  public goForward = () => this.go(1);

  public reset = () => {
    const previousLocation = this.entries[1];
    this.lastAction = 'POP';
    this.index = 0;
    this.notify(previousLocation);
  };

  // Resets the history and reports the second entry as the previous location.
  public resetWith = (nextLocation: LocationDescriptor<S>, nextState?: S) => {
    const previousLocation = this.entries[1];
    this.entries = [this.createLocation(nextLocation, nextState)];
    this.lastAction = 'REPLACE';
    this.index = 0;
    this.notify(previousLocation);
  };

  // Resets the history and reports the current entry as the previous location.
  public resetTo = (nextLocation: LocationDescriptor<S>, nextState?: S) => {
    const previousLocation = this.location;
    this.entries = [this.createLocation(nextLocation, nextState)];
    this.lastAction = 'REPLACE';
    this.index = 0;
    this.notify(previousLocation);
  };

  public resetWithIfDifferent = (nextLocation: LocationDescriptor<S>, nextState?: S) => {
    const location = this.createLocation(nextLocation, nextState);
    if (this.entries[0].pathname !== location.pathname) {
      this.resetWith(nextLocation, nextState);
    }
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

  private notify(previousLocation: Location<S>) {
    this.listeners.forEach((listener) => listener(this.location, this.action, previousLocation));
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
