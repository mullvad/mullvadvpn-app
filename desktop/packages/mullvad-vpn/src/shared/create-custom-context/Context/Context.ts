import createCustomContextWithProvider from '../create-custom-context-with-provider';

export type ComponentContextValues = {
  property: number;
};

const Context = createCustomContextWithProvider<ComponentContextValues, ComponentContextValues>(
  ({ property }) => ({ property }),
);

export const [useComponentContext, { useProperty }, withComponentContextProvider] = Context;
