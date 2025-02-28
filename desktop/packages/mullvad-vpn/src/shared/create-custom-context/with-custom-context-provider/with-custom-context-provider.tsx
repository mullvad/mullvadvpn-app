import React from 'react';

import { ForbiddenProps } from '../types';
import { getFallbackComponentProps, getFallbackContextProviderProps } from './helpers';
import type { Options } from './types';

const withCustomContextProvider = <
  ComponentProps extends object,
  ContextProviderProps extends object,
>(
  Component: React.FunctionComponent<ComponentProps>,
  ContextProvider: React.FunctionComponent<ContextProviderProps>,
  options?: Options<ComponentProps, ContextProviderProps, ComponentProps & ContextProviderProps>,
) => {
  const {
    useGetComponentProps = getFallbackComponentProps,
    useGetContextProviderProps = getFallbackContextProviderProps,
  } = options || {};

  const ComponentWithContextProvider: React.ForwardRefRenderFunction<
    React.ElementRef<typeof Component>,
    ComponentProps & ForbiddenProps<ContextProviderProps>
  > = (props, _ref) => {
    const contextProviderProps = useGetContextProviderProps(props);
    const componentProps = useGetComponentProps(props, contextProviderProps);

    return (
      <ContextProvider {...contextProviderProps}>
        <Component {...componentProps} />
      </ContextProvider>
    );
  };

  return ComponentWithContextProvider;
};

export default withCustomContextProvider;
