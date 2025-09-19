import React from 'react';

export const useIsDefaultFocusOnLoad = (): boolean => {
  const [isDefaultOnLoad, setIsDefaultOnLoad] = React.useState(true);

  React.useEffect(() => {
    if (typeof document === 'undefined') return;
    setIsDefaultOnLoad(
      document.activeElement === document.body ||
        document.activeElement === document.documentElement,
    );
  }, []);

  return isDefaultOnLoad;
};
