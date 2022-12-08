import { Action, History as OriginalHistory, Location, LocationDescriptorObject } from 'history';
import { useHistory as useReactRouterHistory } from 'react-router';

import { IHistoryObject, LocationState } from '../../shared/ipc-types';
import { GeneratedRoutePath, RoutePath } from './routes';

export interface ITransitionSpecification {
  name: string;
  duration: number;
}

interface ITransitionMap {
  [name: string]: ITransitionSpecification;
}

/**
 * Transition descriptors
 */
export const transitions: ITransitionMap = {
  show: {
    name: 'slide-up',
    duration: 450,
  },
  dismiss: {
    name: 'slide-down',
    duration: 450,
  },
  push: {
    name: 'push',
    duration: 450,
  },
  pop: {
    name: 'pop',
    duration: 450,
  },
  none: {
    name: '',
    duration: 0,
  },
};

const transitionOpposites: Record<string, string> = {
  'slide-up': 'slide-down',
  'slide-down': 'slide-up',
  push: 'pop',
  pop: 'push',
  '': '',
};

function oppositeTransition(transition: ITransitionSpecification): ITransitionSpecification {
  return {
    ...transition,
    name: transitionOpposites[transition.name],
  };
}

type LocationDescriptor = RoutePath | GeneratedRoutePath | LocationDescriptorObject<LocationState>;

type LocationListener = (
  location: Location<LocationState>,
  action: Action,
  transition: ITransitionSpecification,
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
    const state = { transition: transitions.push, ...nextState };
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

    this.notify(nextState?.transition ?? transitions.none);
  };

  public listen(callback: LocationListener) {
    this.listeners.push(callback);
    return () => (this.listeners = this.listeners.filter((listener) => listener !== callback));
  }

  public canGo(n: number) {
    const nextIndex = this.index + n;
    return nextIndex >= 0 && nextIndex < this.entries.length;
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

  private popImpl(n = 1): ITransitionSpecification | undefined {
    if (this.canGo(-n)) {
      this.lastAction = 'POP';
      this.index -= n;

      const transition = this.entries[this.index + 1].state.transition;
      this.entries = this.entries.slice(0, this.index + 1);

      return oppositeTransition(transition);
    } else {
      return undefined;
    }
  }

  private notify(transition: ITransitionSpecification) {
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
      transition: state?.transition ?? transitions.none,
    };
  }

  private getRandomKey() {
    return Math.random().toString(36).substr(8);
  }
}

export function useHistory(): History {
  return useReactRouterHistory<LocationState>() as History;
}
