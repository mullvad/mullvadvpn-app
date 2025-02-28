import type { Context } from 'react';

import type { WithCustomContextProvider } from './with-custom-context-provider/types';

export type CustomContextSetValuesCallback<ContextValues extends CustomContextValues> = (
  stateValues: ContextValues,
) => ContextValues;

export type CustomContextSetValues<ContextValues extends CustomContextValues> = (
  values: Partial<ContextValues> | CustomContextSetValuesCallback<ContextValues>,
) => void;

export type CustomContextValues = object | undefined;

export type CustomContext<ContextValues extends CustomContextValues> = {
  values: ContextValues;
  setValues: CustomContextSetValues<ContextValues>;
};

export type CustomContextReact<ContextValues extends CustomContextValues> = Context<
  CustomContext<ContextValues>
>;

export type UseContext<ContextValues extends CustomContextValues> =
  () => CustomContext<ContextValues>;

export type CreateContextWithProvider<
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
> = [
  useContext: UseContext<ContextValues>,
  withCustomContextProvider: <ComponentProps extends object>(
    Component: React.FunctionComponent<ComponentProps>,
  ) => ReturnType<WithCustomContextProvider<ComponentProps, ProviderProps>>,
];
