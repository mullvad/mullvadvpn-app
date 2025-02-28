import { useContext, useEffect } from 'react';

import type { CustomContextReact, CustomContextValues } from '../../../types';
import { useObjectWithContextKeysAndPropsValues, useShouldUpdateValues } from './hooks';

const useFallbackUpdateValues = <
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
>(
  props: ProviderProps,
  Context: CustomContextReact<ContextValues>,
) => {
  const { values, setValues } = useContext(Context);

  const propsValues = useObjectWithContextKeysAndPropsValues(values, props);
  const shouldUpdateValues = useShouldUpdateValues(propsValues);

  useEffect(() => {
    if (shouldUpdateValues) {
      setValues(propsValues);
    }
  }, [setValues, shouldUpdateValues, propsValues]);
};

export default useFallbackUpdateValues;
