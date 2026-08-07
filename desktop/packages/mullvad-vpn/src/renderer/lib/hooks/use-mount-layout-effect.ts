import React from 'react';

export function useMountLayoutEffect(func: () => void) {
  const ref = React.useRef(false);

  React.useLayoutEffect(() => {
    if (!ref.current) {
      func();
      ref.current = true;
    }
  }, [func]);
}
