import omit from 'lodash/omit';

import { getObjectKeys } from '../../get-object-keys';

const getFallbackComponentProps = <
  Props extends object,
  ContextProviderProps extends object,
  ComponentActualProps extends object,
>(
  props: Props,
  contextProviderProps: ContextProviderProps,
) => {
  const forbiddenProps = getObjectKeys(contextProviderProps);
  const componentProps = omit(props, forbiddenProps) as unknown as ComponentActualProps;

  return componentProps;
};

export default getFallbackComponentProps;
