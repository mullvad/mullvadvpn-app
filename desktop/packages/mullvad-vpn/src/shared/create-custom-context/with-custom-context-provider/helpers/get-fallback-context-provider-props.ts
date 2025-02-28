const getFallbackContextProviderProps = <Props extends object, ContextProviderProps extends object>(
  props: Props,
) => {
  return props as unknown as ContextProviderProps;
};

export default getFallbackContextProviderProps;
