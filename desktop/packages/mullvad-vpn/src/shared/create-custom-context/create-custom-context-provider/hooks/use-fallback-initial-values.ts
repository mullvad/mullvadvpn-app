import type { CustomContextValues } from '../../types';

const useFallbackInitialValues = <
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
>(
  props: ProviderProps,
) => {
  const initialValues = {
    ...props,
  } as unknown as ContextValues;

  return initialValues;
};

export default useFallbackInitialValues;
