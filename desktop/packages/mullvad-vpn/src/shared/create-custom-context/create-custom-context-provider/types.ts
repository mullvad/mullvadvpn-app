import type { CustomContextValues } from '../types';

export type ContextProviderProps<ProviderProps extends object> = Array<keyof ProviderProps>;

export type UseInitialValues<
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
> = (props: ProviderProps) => ContextValues;

export type UseUpdateValues<
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
  UpdatedContextValues extends ContextValues,
> = (values: ContextValues, props: ProviderProps) => UpdatedContextValues;
