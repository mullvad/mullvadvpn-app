import type { CustomContextValues } from '../../../types';
import { useObjectWithContextKeysAndPropsValues } from './hooks';

const useFallbackUpdateValues = <
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
  UpdatedContextValues extends CustomContextValues,
>(
  values: ContextValues,
  props: ProviderProps,
): UpdatedContextValues => {
  const propsValues = useObjectWithContextKeysAndPropsValues(values, props);

  return propsValues as UpdatedContextValues;
};

export default useFallbackUpdateValues;
