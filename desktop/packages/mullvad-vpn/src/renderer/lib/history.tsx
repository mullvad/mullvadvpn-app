import { Action, History as OriginalHistory, Location, LocationDescriptorObject } from 'history';
import { useHistory as useReactRouterHistory } from 'react-router';

import { IHistoryObject, LocationState } from '../../shared/ipc-types';
import { GeneratedRoutePath } from './routeHelpers';
import { RoutePath } from './routes';

export enum TransitionType {
  show,
  dismiss,
  push,
  pop,
  none,
}

export interface ITransitionSpecification {
  type: TransitionType;
  duration: number;
}

function oppositeTransition(transition: TransitionType): TransitionType {
  switch (transition) {
    case TransitionType.show:
      return TransitionType.dismiss;
    case TransitionType.dismiss:
      return TransitionType.none;
    case TransitionType.push:
      return TransitionType.pop;
    case TransitionType.pop:
      return TransitionType.none;
    case TransitionType.none:
      return TransitionType.none;
  }
}

type LocationDescriptor = RoutePath | GeneratedRoutePath | LocationDescriptorObject<LocationState>;

type LocationListener = (
  location: Location<LocationState>,
  action: Action,
  transition: TransitionType,
) => void;

export default class History {
  private listeners: LocationListener[] = [];
  private entries: Location<LocationState>[];
  private index = 0;
  private lastAction: Action = 'POP';

  public constructor(location: LocationDescriptor, state?: LocationState) {
    this.entries = [this.createLocation(location, state)];
  }

  public static fromSavedHistory(savedHistory: IHistoryObject): History {
    const history = new History(RoutePath.launch);
    history.entries = savedHistory.entries;
    history.index = savedHistory.index;
    history.lastAction = savedHistory.lastAction;

    return history;
  }

  public get location(): Location<LocationState> {
    return this.entries[this.index];
  }

  public get length(): number {
    return this.entries.length;
  }

  public get action(): Action {
    return this.lastAction;
  }

  public push = (nextLocation: LocationDescriptor, nextState?: Partial<LocationState>) => {
    const state = { transition: TransitionType.push, ...nextState };
    this.pushImpl(nextLocation, state);
    this.notify(state.transition);
  };

  public pop = (all?: boolean) => {
    const transition = this.popImpl(all === true ? this.index : 1);
    if (transition !== undefined) {
      this.notify(transition);
    }
  };

  public reset = (nextLocation: LocationDescriptor, nextState?: Partial<LocationState>) => {
    const location = this.createLocation(nextLocation, nextState);
    this.lastAction = 'REPLACE';
    this.index = 0;
    this.entries = [location];

    this.notify(nextState?.transition ?? TransitionType.none);
  };

  public replaceRoot = (
    replacementLocation: LocationDescriptor,
    replacementState?: Partial<LocationState>,
  ) => {
    const location = this.createLocation(replacementLocation, replacementState);
    this.lastAction = 'REPLACE';
    this.entries.splice(0, 1, location);

    if (this.index === 0) {
      this.notify(replacementState?.transition ?? TransitionType.none);
    }
  };

  public listen(callback: LocationListener) {
    this.listeners.push(callback);
    return () => (this.listeners = this.listeners.filter((listener) => listener !== callback));
  }

  public canGo(n: number) {
    const nextIndex = this.index + n;
    return nextIndex >= 0 && nextIndex < this.entries.length;
  }

  public getPopTransition(steps = 1) {
    // The back transition should be based on the last view to be popped, i.e. the one with the
    // lowest index.
    const transition = this.entries[this.index - steps + 1].state.transition;
    return oppositeTransition(transition);
  }

  // This returns this object casted as History from the History module. The difference between this
  // one and the one in the history module is that this one has stricter types for the paths.
  // Instead of accepting any string it's limited to the paths we actually support. But this history
  // implementation would handle any string as expected.
  public get asHistory(): OriginalHistory {
    return this as OriginalHistory;
  }

  public get asObject(): IHistoryObject {
    return {
      entries: this.entries,
      index: this.index,
      lastAction: this.lastAction,
    };
  }

  public block(): never {
    throw Error('Not implemented');
  }
  public replace(): never {
    throw Error('Not implemented');
  }
  public go(): never {
    throw Error('Not implemented');
  }
  public goBack(): never {
    throw Error('Not implemented');
  }
  public goForward(): never {
    throw Error('Not implemented');
  }
  public createHref(): never {
    throw Error('Not implemented');
  }

  private pushImpl(nextLocation: LocationDescriptor, nextState?: Partial<LocationState>) {
    const location = this.createLocation(nextLocation, nextState);
    this.lastAction = 'PUSH';
    this.index += 1;
    this.entries.splice(this.index, this.entries.length - this.index, location);
  }

  private popImpl(n = 1): TransitionType | undefined {
    if (this.canGo(-n)) {
      const transition = this.getPopTransition(n);

      this.lastAction = 'POP';
      this.index -= n;
      this.entries = this.entries.slice(0, this.index + 1);

      return transition;
    } else {
      return undefined;
    }
  }

  private notify(transition: TransitionType) {
    this.listeners.forEach((listener) => listener(this.location, this.action, transition));
  }

  private createLocation(
    location: LocationDescriptor,
    state?: Partial<LocationState>,
  ): Location<LocationState> {
    if (typeof location === 'string') {
      return this.createLocationFromString(location, state);
    } else if ('routePath' in location) {
      return this.createLocationFromString(location.routePath, state);
    } else {
      return {
        pathname: location.pathname ?? this.location.pathname,
        search: location.search ?? '',
        hash: location.hash ?? '',
        state: this.createState(state),
        key: location.key ?? this.getRandomKey(),
      };
    }
  }

  private createLocationFromString(
    path: string,
    state?: Partial<LocationState>,
  ): Location<LocationState> {
    return {
      pathname: path,
      search: '',
      hash: '',
      state: this.createState(state),
      key: this.getRandomKey(),
    };
  }

  private createState(state?: Partial<LocationState>): LocationState {
    return {
      scrollPosition: state?.scrollPosition ?? [0, 0],
      expandedSections: state?.expandedSections ?? {},
      transition: state?.transition ?? TransitionType.none,
    };
  }

  private getRandomKey() {
    return Math.random().toString(36).substr(8);
  }
}

export function useHistory(): History {
  return useReactRouterHistory<LocationState>() as History;
}
