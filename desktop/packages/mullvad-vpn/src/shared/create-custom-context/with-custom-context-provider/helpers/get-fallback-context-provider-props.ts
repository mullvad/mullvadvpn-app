import pick from 'lodash/pick';

const getFallbackContextProviderProps = <Props extends object, ContextProviderProps extends object>(
  props: Props,
  contextProviderProps: Array<keyof ContextProviderProps>,
) => {
  return pick(props, contextProviderProps) as unknown as ContextProviderProps;
};

export default getFallbackContextProviderProps;
