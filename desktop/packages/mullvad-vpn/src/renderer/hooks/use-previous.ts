import React from 'react';

export function usePrevious<T>(value: T): T {
  const [previous, setPrevious] = React.useState(value);

  React.useEffect(() => setPrevious(value), [value]);

  return previous;
}
