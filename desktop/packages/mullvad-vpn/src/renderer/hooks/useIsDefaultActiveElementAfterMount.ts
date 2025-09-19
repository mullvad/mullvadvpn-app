import React from 'react';

export const useIsDefaultActiveElementAfterMount = () => {
  const [isDefaultActiveElementAfterMount, setIsDefaultActiveElementAfterMount] = React.useState<
    boolean | undefined
  >(undefined);

  React.useEffect(() => {
    if (typeof document !== 'undefined') {
      const isBodyOrDocumentElement =
        document.activeElement === document.body ||
        document.activeElement === document.documentElement;

      setIsDefaultActiveElementAfterMount(isBodyOrDocumentElement);
    }

    return () => {
      setIsDefaultActiveElementAfterMount(undefined);
    };
  }, []);

  return isDefaultActiveElementAfterMount;
};
