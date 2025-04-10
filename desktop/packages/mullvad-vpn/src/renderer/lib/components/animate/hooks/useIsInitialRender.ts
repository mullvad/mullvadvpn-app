import React from 'react';

export const useIsInitialRender = () => {
  const isInitialRender = React.useRef(true);

  React.useEffect(() => {
    isInitialRender.current = false;
  }, []);

  return React.useCallback(() => isInitialRender.current, [isInitialRender]);
};
