import React from 'react';

import { useEffectEvent } from '../lib/utility-hooks';

export const useIsDefaultActiveElementAfterMount = () => {
  const [isDefaultActiveElementAfterMount, setIsDefaultActiveElementAfterMount] = React.useState<
    boolean | undefined
  >(undefined);

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
