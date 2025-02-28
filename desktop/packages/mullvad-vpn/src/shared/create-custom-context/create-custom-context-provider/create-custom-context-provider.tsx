import React from 'react';

import type { CustomContextReact, CustomContextValues } from '../types';
import { useFallbackInitialValues, useFallbackUpdateValues, useInitialContext } from './hooks';
import type { UseInitialValues, UseUpdateValues } from './types';

const createCustomContextProvider = <
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
>(
  Context: CustomContextReact<ContextValues>,
  useInitialValues: UseInitialValues<
    Partial<ContextValues>,
    ProviderProps
  > = useFallbackInitialValues,
  useUpdateValues: UseUpdateValues<
    ContextValues,
    Omit<ProviderProps, 'children'>
  > = useFallbackUpdateValues,
) => {
  const ComponentWithContextEffects = ({
    children,
    ...props
  }: React.PropsWithChildren<ProviderProps>) => {
    useUpdateValues(props, Context);

    return <>{children}</>;
  };

  const CustomContextProvider: React.FunctionComponent<ProviderProps> = (props) => {
    const initialValues: Partial<ContextValues> = useInitialValues(props);
    const context = useInitialContext<ContextValues>(initialValues as ContextValues);

    return (
      <Context.Provider value={context}>
        <ComponentWithContextEffects {...props} />
      </Context.Provider>
    );
  };

  return CustomContextProvider;
};

export default createCustomContextProvider;
