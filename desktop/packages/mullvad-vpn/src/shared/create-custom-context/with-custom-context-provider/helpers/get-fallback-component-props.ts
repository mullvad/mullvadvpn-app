import omit from 'lodash/omit';

const getFallbackComponentProps = <
  Props extends object,
  ContextProviderProps extends object,
  ComponentProps extends object,
>(
  props: Props,
  contextProviderProps: Array<keyof ContextProviderProps>,
) => {
  const componentProps = omit(props, contextProviderProps) as unknown as ComponentProps;

  return componentProps;
};

export default getFallbackComponentProps;
