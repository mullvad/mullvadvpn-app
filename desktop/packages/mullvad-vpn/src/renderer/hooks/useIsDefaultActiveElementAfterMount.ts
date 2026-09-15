import React from 'react';

import { useEffectEvent } from '../lib/utility-hooks';

export const useIsDefaultActiveElementAfterMount = () => {
  const [isDefaultActiveElementAfterMount, setIsDefaultActiveElementAfterMount] = React.useState<
    boolean | undefined
  >(undefined);

  // TODO: Remove the use of useEffectEvent. This is used as an escape hatch
  // in order to be able to continue setting state from a useEffect without
  // lint errors.
  //
  // The entire logic should be rewritten to no longer depend on setting
  // state from an effect.
  const setIsDefaultActiveElementAfterMountEffectEvent = useEffectEvent((value: boolean) => {
    setIsDefaultActiveElementAfterMount(value);
  });

  React.useEffect(() => {
    if (typeof document !== 'undefined') {
      const isBodyOrDocumentElement =
        document.activeElement === document.body ||
        document.activeElement === document.documentElement;

      setIsDefaultActiveElementAfterMountEffectEvent(isBodyOrDocumentElement);
    }

    return () => {
      setIsDefaultActiveElementAfterMount(undefined);
    };
  }, []);

  return isDefaultActiveElementAfterMount;
};
