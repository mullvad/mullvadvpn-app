export type GetComponentProps<
  ComponentProps extends object,
  ContextProviderProps extends object,
  Props extends ContextProviderProps,
> = (props: Props, contextProviderProps: ContextProviderProps) => ComponentProps;

export type GetContextProviderProps<Props extends object, ContextProviderProps extends object> = (
  props: Props,
) => ContextProviderProps;

export type Options<
  ComponentProps extends object,
  ContextProviderProps extends object,
  Props extends ContextProviderProps,
> = {
  useGetComponentProps?: GetComponentProps<ComponentProps, ContextProviderProps, Props>;
  useGetContextProviderProps?: GetContextProviderProps<Props, ContextProviderProps>;
};
