import createCustomContext from './create-custom-context';
import withCustomContext from './create-custom-context-provider';
import type {
  ContextProviderProps,
  UseInitialValues,
  UseUpdateValues,
} from './create-custom-context-provider/types';
import createUseCustomContext from './create-use-custom-context';
import type { CreateContextWithProvider, CustomContextValues } from './types';
import withCustomContextProvider from './with-custom-context-provider';

const createCustomContextWithProvider = <
  Values extends CustomContextValues,
  ProviderProps extends object,
>(
  contextProviderProps: ContextProviderProps<ProviderProps>,
  useInitialValues?: UseInitialValues<Values, ProviderProps>,
  useUpdateValues?: UseUpdateValues<Values, Omit<ProviderProps, 'children'>, Values>,
): CreateContextWithProvider<Values, ProviderProps> => {
  const Context = createCustomContext<Values>();

  const useCustomContext = createUseCustomContext<Values>(Context);

  const CustomContextProvider = withCustomContext<Values, ProviderProps>(
    Context,
    useInitialValues,
    useUpdateValues,
  );

  const withContextProvider = <ComponentProps extends object>(
    Component: React.FunctionComponent<ComponentProps>,
  ) => {
    const ComponentWithContextProvider = withCustomContextProvider<
      ComponentProps,
      ProviderProps,
      ComponentProps & ProviderProps
    >(Component, CustomContextProvider, contextProviderProps);

    return ComponentWithContextProvider;
  };

  return [useCustomContext, withContextProvider];
};

export default createCustomContextWithProvider;
