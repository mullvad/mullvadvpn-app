import createCustomContext from '../../../shared/create-custom-context';

export type Values = {
  count: number;
  initialCount: number;
};

export type Props = {
  initialCount: number;
};

const Context = createCustomContext<Values, Props>(
  // Get props which should be passed to the context state builder hooks
  ['initialCount'],
  // Build initial context state
  ({ initialCount }) => {
    const initialState: Values = {
      count: initialCount,
      initialCount,
    };

    return initialState;
  },
);

export const [useCustomComponentContext, withComponentContextProvider] = Context;
