export type ContextProviderProps<Props extends object, ProviderProps extends object> = (
  props: Props,
) => ProviderProps;

export type WithCustomContextProvider<
  ComponentProps extends object,
  ProviderProps extends object,
> = (
  Component: React.FunctionComponent<ComponentProps>,
  ContextProvider: React.FunctionComponent<ProviderProps>,
  contextProviderProps?: Array<keyof ProviderProps>,
) => React.FunctionComponent<ComponentProps & ProviderProps>;
