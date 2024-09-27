import { Location } from 'history';
import { useCallback, useEffect, useRef, useState } from 'react';
import { flushSync } from 'react-dom';

import { ViewTransition } from '../../../types/global';
import { LocationState } from '../../shared/ipc-types';
import { useAppContext } from '../context';
import { TransitionType, useHistory } from '../lib/history';
import { useEffectEvent } from './utility-hooks';

type QueueItem = { location: Location<LocationState>; transition: TransitionType };

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

export function useViewTransitions(): Location<LocationState> {
  const history = useHistory();
  const [currentLocation, setCurrentLocation] = useState(history.location);
  const queueLocationRef = useRef<QueueItem | undefined>();
  const { setNavigationHistory } = useAppContext();

  const transitionToView = useEffectEvent(
    (location: Location<LocationState>, transition: TransitionType) => {
      flushSync(() => {
        viewTransitionRef.current = document.startViewTransition(() => {
          setNavigationHistory(history.asObject);
          setCurrentLocation(location);
        });

        void viewTransitionRef.current.ready.then(() => animateNavigation(transition));
        void viewTransitionRef.current.finished.then(() => {
          const queueLocation = queueLocationRef.current;

          delete viewTransitionRef.current;
          delete queueLocationRef.current;

          if (queueLocation) {
            transitionToView(queueLocation.location, queueLocation.transition);
          }
        });
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
        queueLocationRef.current = { location, transition };
      }
    });

    return () => {
      unobserveHistory?.();
    };
  }, [history]);

  return currentLocation;
}

function animateNavigation(transition: TransitionType) {
  const oldInFront = transition === TransitionType.dismiss || transition === TransitionType.pop;
  const oldZIndex = oldInFront ? 2 : 0;

  const boxShadow = '0 0 300px 50px rgba(0, 0, 0, 0.7)';

  document.documentElement.animate(
    [
      { transform: 'translate(0%, 0%)', zIndex: oldZIndex, boxShadow },
      { transform: oldToTransform[transition], zIndex: oldZIndex },
    ],
    {
      duration: 450,
      easing: 'ease-in-out',
      pseudoElement: '::view-transition-old(root)',
    },
  );
  document.documentElement.animate(
    [{ transform: newFromTransform[transition] }, { transform: 'translate(0%, 0%)', boxShadow }],
    {
      duration: 450,
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
