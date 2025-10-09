import { Location } from 'history';
import { useCallback, useEffect, useRef, useState } from 'react';

import { ViewTransition } from '../../../types/global';
import { LocationState } from '../../shared/ipc-types';
import { useAppContext } from '../context';
import { TransitionType, useHistory } from '../lib/history';
import { getReduceMotion } from './functions';
import { useEffectEvent } from './utility-hooks';

type QueueItem = { location: Location<LocationState>; transition: TransitionType };

const TRANSITION_DURATION = 450;

const viewTransitionRef: { current?: ViewTransition } = {};

export function useAfterTransition() {
  const runAfterTransition = useCallback((fn: () => void) => {
    if (viewTransitionRef.current) {
      void viewTransitionRef.current.finished.then(() => runAfterTransition(fn));
    } else {
      fn();
    }
  }, []);

  return runAfterTransition;
}

export function useViewTransitions(onTransition?: () => void): Location<LocationState> {
  const history = useHistory();
  const [currentLocation, setCurrentLocation] = useState(history.location);
  const queuedLocationRef = useRef<QueueItem>(undefined);
  const { setNavigationHistory } = useAppContext();

  const updateView = useEffectEvent((location: Location<LocationState>) => {
    setCurrentLocation(location);
    setNavigationHistory(history.asObject);
  });

  const onTransitionEnd = useEffectEvent((location: Location<LocationState>) => {
    if (window.env.e2e) {
      window.e2e.location = location.pathname;
    }

    onTransition?.();
  });

  const transitionToView = useEffectEvent(
    (location: Location<LocationState>, transition: TransitionType) => {
      if (getReduceMotion()) {
        updateView(location);
        setTimeout(() => onTransitionEnd(location));
        return;
      }

      viewTransitionRef.current = document.startViewTransition(() => {
        updateView(location);
      });

      void viewTransitionRef.current.ready.then(() => animateNavigation(transition));
      void viewTransitionRef.current.finished.then(() => {
        const queueLocation = queuedLocationRef.current;

        delete viewTransitionRef.current;
        delete queuedLocationRef.current;

        if (queueLocation) {
          transitionToView(queueLocation.location, queueLocation.transition);
        } else {
          onTransitionEnd?.(location);
        }
      });
    },
  );

  useEffect(() => {
    // React throttles updates, so it's impossible to capture the intermediate navigation without
    // listening to the history directly.
    const unobserveHistory = history.listen((location, _, transition) => {
      if (viewTransitionRef.current === undefined) {
        transitionToView(location, transition);
      } else {
        queuedLocationRef.current = { location, transition };
      }
    });

    return () => {
      unobserveHistory?.();
    };
    // These lint rules are disabled for now because the react plugin for eslint does
    // not understand that useEffectEvent should not be added to the dependency array.
    // Enable these rules again when eslint can lint useEffectEvent properly.
    // eslint-disable-next-line react-compiler/react-compiler
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [history]);

  return currentLocation;
}

function animateNavigation(transition: TransitionType) {
  const oldInFront = transition === TransitionType.dismiss || transition === TransitionType.pop;
  const oldZIndex = oldInFront ? 2 : 0;

  document.documentElement.animate(
    [
      { transform: 'translate(0%, 0%)', zIndex: oldZIndex },
      { transform: oldToTransform[transition], zIndex: oldZIndex },
    ],
    {
      duration: TRANSITION_DURATION,
      easing: 'ease-in-out',
      pseudoElement: '::view-transition-old(root)',
    },
  );
  document.documentElement.animate(
    [{ transform: newFromTransform[transition] }, { transform: 'translate(0%, 0%)' }],
    {
      duration: TRANSITION_DURATION,
      easing: 'ease-in-out',
      pseudoElement: '::view-transition-new(root)',
    },
  );
}

const oldToTransform = {
  [TransitionType.show]: 'translateY(0%)',
  [TransitionType.dismiss]: 'translateY(100%)',
  [TransitionType.push]: 'translateX(-33%)',
  [TransitionType.pop]: 'translateX(100%)',
  [TransitionType.none]: '',
};

const newFromTransform = {
  [TransitionType.show]: 'translateY(100%)',
  [TransitionType.dismiss]: 'translateY(0%)',
  [TransitionType.push]: 'translateX(100%)',
  [TransitionType.pop]: 'translateX(-33%)',
  [TransitionType.none]: '',
};
