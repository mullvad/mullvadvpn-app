import React from 'react';

import { getFallbackComponentProps, getFallbackContextProviderProps } from './helpers';

const withCustomContextProvider = <
  ComponentProps extends object,
  ProviderProps extends object,
  CombinedProps extends object,
>(
  Component: React.FunctionComponent<ComponentProps>,
  ContextProvider: React.FunctionComponent<ProviderProps>,
  contextProviderProps: Array<keyof ProviderProps> = [],
) => {
  const ComponentWithContextProvider: React.FunctionComponent<CombinedProps> = (props) => {
    const providerProps = getFallbackContextProviderProps(props, contextProviderProps);
    const componentProps = getFallbackComponentProps<CombinedProps, ProviderProps, ComponentProps>(
      props,
      contextProviderProps,
    );

    return (
      <ContextProvider {...providerProps}>
        <Component {...componentProps} />
      </ContextProvider>
    );
  };

  return ComponentWithContextProvider;
};

export default withCustomContextProvider;
