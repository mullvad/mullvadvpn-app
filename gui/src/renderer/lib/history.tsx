import { Location, Action, LocationDescriptorObject, History as OriginalHistory } from 'history';
import React from 'react';
import { RouteComponentProps, useHistory as useReactRouterHistory, withRouter } from 'react-router';
import { RoutePath } from './routes';

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

type LocationDescriptor<S> = RoutePath | LocationDescriptorObject<S>;

type LocationListener<S = unknown> = (
  location: Location<S>,
  action: Action,
  transition: ITransitionSpecification,
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

  public constructor(location: LocationDescriptor<S>, state?: S) {
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
    this.pushImpl(nextLocation, nextState);
    this.notify(transitions.push);
  };

  public pop = () => {
    if (this.popImpl()) {
      this.notify(transitions.pop);
    }
  };

  public show = (nextLocation: LocationDescriptor<S>, nextState?: S) => {
    this.pushImpl(nextLocation, nextState);
    this.notify(transitions.show);
  };

  public dismiss = (all?: boolean, transition = transitions.dismiss) => {
    if (this.popImpl(all ? this.index : 1)) {
      this.notify(transition);
    }
  };

  public reset = (
    nextLocation: LocationDescriptor<S>,
    transition?: ITransitionSpecification,
    nextState?: S,
  ) => {
    const location = this.createLocation(nextLocation, nextState);
    this.lastAction = 'REPLACE';
    this.index = 0;
    this.entries = [location];

    this.notify(transition ?? transitions.none);
  };

  public listen(callback: LocationListener<S>) {
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

  private pushImpl(nextLocation: LocationDescriptor<S>, nextState?: S) {
    const location = this.createLocation(nextLocation, nextState);
    this.lastAction = 'PUSH';
    this.index += 1;
    this.entries.splice(this.index, this.entries.length - this.index, location);
  }

  private popImpl(n = 1): boolean {
    if (this.canGo(-n)) {
      this.lastAction = 'POP';
      this.index -= n;
      this.entries = this.entries.slice(0, this.index + 1);

      return true;
    } else {
      return false;
    }
  }

  private notify(transition: ITransitionSpecification) {
    this.listeners.forEach((listener) => listener(this.location, this.action, transition));
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

export function useHistory(): History {
  return useReactRouterHistory() as History;
}

export interface IHistoryProps {
  history: History;
}

export function withHistory<P>(BaseComponent: React.ComponentType<P & IHistoryProps>) {
  return withRouter((props: P & RouteComponentProps) => {
    const history = props.history as History;
    const mergedProps = ({ ...props, history } as unknown) as P & IHistoryProps;
    return <BaseComponent {...mergedProps} />;
  });
}
