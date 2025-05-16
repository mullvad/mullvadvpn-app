import { useEffect, useState } from 'react';

export const usePreviousValue = <T>(value: T) => {
  const [previousValue, setPreviousValue] = useState(value);

  useEffect(() => {
    setPreviousValue(value);
  }, [setPreviousValue, value]);

  return previousValue;
};
