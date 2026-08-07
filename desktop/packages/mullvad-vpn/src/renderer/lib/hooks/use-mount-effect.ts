import React from 'react';

export function useMountEffect(func: () => void) {
  const ref = React.useRef(false);

  React.useEffect(() => {
    if (!ref.current) {
      func();
      ref.current = true;
    }
  }, [func]);
}
