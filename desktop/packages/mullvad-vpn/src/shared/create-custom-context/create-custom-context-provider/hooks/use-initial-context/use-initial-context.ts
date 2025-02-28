import { useMemo, useState } from 'react';

import type { CustomContext, CustomContextValues } from '../../../types';
import { useCustomContextSetValues } from './hooks';

const useInitialContext = <ContextValues extends CustomContextValues>(
  initialValues: ContextValues,
) => {
  const [values, setValues] = useState<ContextValues>(initialValues);
  const setContextValues = useCustomContextSetValues(setValues);

  const context: CustomContext<ContextValues> = useMemo(
    () => ({
      values,
      setValues: setContextValues,
    }),
    [values, setContextValues],
  );

  return context;
};

export default useInitialContext;
