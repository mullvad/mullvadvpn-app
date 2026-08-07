import React from 'react';

export function useOnStableLayout(func: () => void) {
  const [isLayoutStable, setIsLayoutStable] = React.useState(false);

  React.useLayoutEffect(() => {
    setIsLayoutStable(true);
  }, []);

  React.useLayoutEffect(() => {
    if (!isLayoutStable) {
      func();
    }
  }, [isLayoutStable, func]);
}
