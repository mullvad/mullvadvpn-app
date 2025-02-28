import createCustomContext from './create-custom-context';
import createCustomContextHooks from './create-custom-context-hooks';
import withCustomContext from './create-custom-context-provider';
import type {
  GetContextProviderProps,
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
  getContextProviderProps: GetContextProviderProps<ProviderProps>,
  useInitialValues?: UseInitialValues<Values, ProviderProps>,
  useUpdateValues?: UseUpdateValues<Values, Omit<ProviderProps, 'children'>>,
): CreateContextWithProvider<Values, ProviderProps> => {
  const Context = createCustomContext<Values>();

  const useCustomContext = createUseCustomContext<Values>(Context);

  const CustomContextProvider = withCustomContext<Values, ProviderProps>(
    Context,
    useInitialValues,
    useUpdateValues,
  );

  const withContextProvider = <
    ComponentProps extends object,
    ReactComponent extends React.FunctionComponent<ComponentProps>,
  >(
    Component: ReactComponent,
  ) => {
    const ComponentWithContextProvider = withCustomContextProvider<ComponentProps, ProviderProps>(
      Component,
      CustomContextProvider,
      {
        useGetContextProviderProps: getContextProviderProps,
      },
    );

    return ComponentWithContextProvider;
  };

  const customContextHooks = createCustomContextHooks(Context, getContextProviderProps);

  return [useCustomContext, customContextHooks, withContextProvider, CustomContextProvider];
};

export default createCustomContextWithProvider;
