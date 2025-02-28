import React, { useContext, useEffect } from 'react';

import type { CustomContextReact, CustomContextValues } from '../types';
import {
  useFallbackInitialValues,
  useFallbackUpdateValues,
  useInitialContext,
  useShouldUpdateValues,
} from './hooks';
import type { UseInitialValues, UseUpdateValues } from './types';

const createCustomContextProvider = <
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
>(
  Context: CustomContextReact<ContextValues>,
  useInitialValues: UseInitialValues<ContextValues, ProviderProps> = useFallbackInitialValues,
  useUpdateValues: UseUpdateValues<
    ContextValues,
    Omit<ProviderProps, 'children'>,
    ContextValues
  > = useFallbackUpdateValues,
) => {
  const ComponentWithContextEffects = ({
    children,
    ...props
  }: React.PropsWithChildren<ProviderProps>) => {
    const { values, setValues } = useContext(Context);
    const updateValues = useUpdateValues(values, props);
    const shouldUpdateValues = useShouldUpdateValues(updateValues);

    useEffect(() => {
      if (shouldUpdateValues) {
        setValues(updateValues);
      }
    }, [setValues, shouldUpdateValues, updateValues]);

    return <>{children}</>;
  };

  const CustomContextProvider: React.FunctionComponent<ProviderProps> = (props) => {
    const initialValues = useInitialValues(props);
    const context = useInitialContext<ContextValues>(initialValues);

    return (
      <Context.Provider value={context}>
        <ComponentWithContextEffects {...props} />
      </Context.Provider>
    );
  };

  return CustomContextProvider;
};

export default createCustomContextProvider;
