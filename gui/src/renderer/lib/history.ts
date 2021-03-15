import { Location, Action, LocationDescriptor } from 'history';

type LocationListener<S = unknown> = (
  location: Location<S>,
  action: Action,
  entries: Location<S>[],
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
    const affectedEntries = [this.entries[this.index]];
    const location = this.createLocation(nextLocation, nextState);
    this.lastAction = 'PUSH';
    this.index += 1;
    this.entries.splice(this.index, this.entries.length - this.index, location);
    this.notify(affectedEntries);
  };

  public replace = (nextLocation: LocationDescriptor<S>, nextState?: S) => {
    const affectedEntries = [this.entries[this.index]];
    this.entries[this.index] = this.createLocation(nextLocation, nextState);
    this.lastAction = 'REPLACE';
    this.notify(affectedEntries);
  };

  public go = (n: number) => {
    if (this.canGo(n)) {
      const nextIndex = this.index + n;
      const affectedEntries =
        this.index < nextIndex
          ? this.entries.slice(this.index, nextIndex)
          : this.entries.slice(nextIndex + 1, this.index + 1);

      this.index = nextIndex;
      this.lastAction = 'POP';
      this.notify(affectedEntries);
    }
  };

  public goBack = () => this.go(-1);
  public goForward = () => this.go(1);

  public reset = () => {
    const affectedEntries = this.entries.slice(1);
    this.lastAction = 'POP';
    this.index = 0;
    this.notify(affectedEntries);
  };

  public resetWith = (nextLocation: LocationDescriptor<S>, nextState?: S) => {
    const affectedEntries = [...this.entries];
    this.entries = [this.createLocation(nextLocation, nextState)];
    this.lastAction = 'REPLACE';
    this.index = 0;
    this.notify(affectedEntries);
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

  private notify(affectedEntries: Location<S>[]) {
    this.listeners.forEach((listener) => listener(this.location, this.action, affectedEntries));
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
