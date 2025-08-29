import React, { useCallback, useContext, useEffect, useLayoutEffect, useRef } from 'react';

import { useAppContext } from '../context';
import { useHistory } from '../lib/history';
import { useCombinedRefs, useEffectEvent } from '../lib/utility-hooks';
import CustomScrollbars, { CustomScrollbarsRef, IScrollEvent } from './CustomScrollbars';
import { NavigationScrollContext } from './NavigationContainer';

export interface NavigationScrollbarsProps {
  className?: string;
  fillContainer?: boolean;
  children?: React.ReactNode;
}

export const NavigationScrollbars = React.forwardRef(function NavigationScrollbarsT(
  props: NavigationScrollbarsProps,
  forwardedRef?: React.Ref<CustomScrollbarsRef>,
) {
  const history = useHistory();
  const location = useRef(history.location);
  const { setNavigationHistory } = useAppContext();
  const { onScroll } = useContext(NavigationScrollContext);

  const ref = useRef<CustomScrollbarsRef>(undefined);
  const combinedRefs = useCombinedRefs(forwardedRef, ref);

  const beforeunload = useEffectEvent(() => {
    if (ref.current) {
      location.current.state.scrollPosition = ref.current.getScrollPosition();
      setNavigationHistory(history.asObject);
    }
  });

  useEffect(() => {
    window.addEventListener('beforeunload', beforeunload);
    return () => window.removeEventListener('beforeunload', beforeunload);
    // These lint rules are disabled for now because the react plugin for eslint does
    // not understand that useEffectEvent should not be added to the dependency array.
    // Enable these rules again when eslint can lint useEffectEvent properly.
    // eslint-disable-next-line react-compiler/react-compiler
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const onMount = useEffectEvent(() => {
    const location = history.location;
    if (history.action === 'POP') {
      ref.current?.scrollTo(...location.state.scrollPosition);
    }
  });

  const onUnmount = useEffectEvent(() => {
    if (history.action === 'PUSH' && ref.current) {
      location.current.state.scrollPosition = ref.current.getScrollPosition();
      setNavigationHistory(history.asObject);
    }
  });

  useLayoutEffect(() => {
    onMount();
    return () => onUnmount();
    // These lint rules are disabled for now because the react plugin for eslint does
    // not understand that useEffectEvent should not be added to the dependency array.
    // Enable these rules again when eslint can lint useEffectEvent properly.
    // eslint-disable-next-line react-compiler/react-compiler
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleScroll = useCallback(
    (event: IScrollEvent) => {
      onScroll(event);
    },
    [onScroll],
  );

  return (
    <CustomScrollbars
      ref={combinedRefs}
      className={props.className}
      fillContainer={props.fillContainer}
      onScroll={handleScroll}>
      {props.children}
    </CustomScrollbars>
  );
});
