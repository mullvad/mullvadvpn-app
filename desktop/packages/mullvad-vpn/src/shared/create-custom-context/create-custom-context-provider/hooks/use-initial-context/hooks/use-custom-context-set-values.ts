import type { Dispatch, SetStateAction } from 'react';
import { useCallback } from 'react';

import type { CustomContextSetValues, CustomContextValues } from '../../../../types';

const useCustomContextSetValues = <ContextValues extends CustomContextValues>(
  setValues: Dispatch<SetStateAction<ContextValues>>,
) => {
  const setContextValues: CustomContextSetValues<ContextValues> = useCallback(
    (values) => {
      if (typeof values === 'function') {
        setValues((stateValues) => ({
          ...stateValues,
          ...values(stateValues),
        }));
      } else {
        setValues((stateValues) => ({
          ...stateValues,
          ...values,
        }));
      }
    },
    [setValues],
  );

  return setContextValues;
};

export default useCustomContextSetValues;
