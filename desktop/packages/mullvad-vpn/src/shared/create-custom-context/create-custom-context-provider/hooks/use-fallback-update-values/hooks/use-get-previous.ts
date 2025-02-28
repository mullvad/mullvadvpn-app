import { useEffect, useRef } from 'react';

const useGetPrevious = <Value>(value: Value) => {
  const ref = useRef<Value | undefined>(undefined);

  useEffect(() => {
    ref.current = value;
  });

  const getPrevious = () => {
    return ref.current;
  };

  return getPrevious;
};

export default useGetPrevious;
