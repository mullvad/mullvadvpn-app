import type { CustomContextReact, CustomContextValues } from '../types';

export type GetContextProviderProps<ProviderProps extends object> = <Props extends ProviderProps>(
  props: Props,
) => ProviderProps;

export type UseInitialValues<
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
> = (props: ProviderProps) => ContextValues;

export type UseUpdateValues<
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
> = (props: ProviderProps, Context: CustomContextReact<ContextValues>) => void;
