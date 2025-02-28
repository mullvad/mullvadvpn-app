import { useMemo } from 'react';

import { getObjectKeys } from '../../../../get-object-keys';

const useObjectWithContextKeysAndPropsValues = <
  ContextValues extends object,
  ProviderProps extends ContextValues,
>(
  values: ContextValues | undefined,
  props: ProviderProps,
) => {
  const objectWithContextKeysAndPropsValues = useMemo(() => {
    const defaultValue: Partial<ContextValues> = {};
    if (!values) {
      return defaultValue;
    }

    return getObjectKeys(values).reduce((allValues, key) => {
      if (key in props) {
        const value = props[key];

        return {
          ...allValues,
          [key]: value,
        };
      }

      return allValues;
    }, defaultValue);
  }, [values, props]);

  return objectWithContextKeysAndPropsValues;
};

export default useObjectWithContextKeysAndPropsValues;
