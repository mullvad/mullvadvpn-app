import { useContext } from 'react';

import type { CustomContext, CustomContextReact, CustomContextValues } from './types';

const createCustomContextHook = <
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
>(
  Context: CustomContextReact<ContextValues>,
  key: keyof ProviderProps,
) => {
  const useCustomContextHook = () => {
    const { values } = useContext<CustomContext<ContextValues>>(Context);
    const { [key]: value } = values;

    return value;
  };

  return useCustomContextHook;
};

export default createCustomContextHook;
