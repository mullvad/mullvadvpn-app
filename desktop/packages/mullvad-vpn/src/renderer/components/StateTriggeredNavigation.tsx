import { useEffect, useMemo, useRef } from 'react';

import { RoutePath } from '../../shared/routes';
import { useScheduler } from '../../shared/scheduler';
import { getNavigationBase } from '../lib/functions/navigation-base';
import { TransitionType, useHistory } from '../lib/history';
import { useEffectEvent } from '../lib/utility-hooks';
import { useSelector } from '../redux/store';

export default function StateTriggeredNavigation() {
  const { location, reset } = useHistory();

  const connectedToDaemon = useSelector((state) => state.userInterface.connectedToDaemon);
  const hasIdentity = useSelector((state) => state.identity.identity !== undefined);

  const delayScheduler = useScheduler();

  const prevPath = useRef<RoutePath>(getNavigationBase(connectedToDaemon, hasIdentity));
  const nextPath = useMemo(
    () => getNavigationBase(connectedToDaemon, hasIdentity),
    [connectedToDaemon, hasIdentity],
  );

  const updatePath = useEffectEvent((nextPath: RoutePath) => {
    const currentPath = location.pathname as RoutePath;

    if (currentPath !== nextPath) {
      delayScheduler.cancel();

      const transition = getNavigationTransition(currentPath, nextPath);
      const delay = getNavigationDelay(currentPath, nextPath);

      const navigate = () => {
        reset(nextPath, { transition });
      };

      if (delay) {
        delayScheduler.schedule(navigate, delay);
      } else {
        navigate();
      }
    }
  });

  useEffect(() => {
    if (nextPath !== prevPath.current) {
      prevPath.current = nextPath;
      updatePath(nextPath);
    }
  }, [nextPath]);

  return null;
}

function getNavigationDelay(currentPath: RoutePath, nextPath: RoutePath): number | void {
  if (currentPath === RoutePath.importConfig && nextPath === RoutePath.main) {
    return 1000;
  }
}

function getNavigationTransition(currentPath: RoutePath, nextPath: RoutePath) {
  // First level contains the possible next locations and the second level contains the
  // possible current locations.
  const navigationTransitions: Partial<
    Record<RoutePath, Partial<Record<RoutePath | '*', TransitionType>>>
  > = {
    [RoutePath.launch]: {
      [RoutePath.importConfig]: TransitionType.pop,
      [RoutePath.main]: TransitionType.pop,
      '*': TransitionType.dismiss,
    },
    [RoutePath.importConfig]: {
      [RoutePath.launch]: TransitionType.push,
      [RoutePath.main]: TransitionType.pop,
      '*': TransitionType.dismiss,
    },
    [RoutePath.main]: {
      [RoutePath.launch]: TransitionType.push,
      [RoutePath.importConfig]: TransitionType.push,
      '*': TransitionType.dismiss,
    },
  };

  return navigationTransitions[nextPath]?.[currentPath] ?? navigationTransitions[nextPath]?.['*'];
}
