import type { CustomContextValues } from '../../types';

const useFallbackInitialValues = <
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
>(
  props: ProviderProps,
) => {
  const defaultInitialValues = {
    ...props,
  } as unknown as ContextValues;

  return defaultInitialValues;
};

export default useFallbackInitialValues;
