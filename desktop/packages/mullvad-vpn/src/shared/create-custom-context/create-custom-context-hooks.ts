import createCustomContextHook from './create-custom-context-hook';
import createCustomContextHookName from './create-custom-context-hook-name';
import type { GetContextProviderProps } from './create-custom-context-provider/types';
import { getObjectKeys } from './get-object-keys';
import type { CustomContextHooks, CustomContextReact, CustomContextValues } from './types';

const createCustomContextHooks = <
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
>(
  Context: CustomContextReact<ContextValues>,
  getProviderProps: GetContextProviderProps<ProviderProps>,
) => {
  const customContextProviderProps = getProviderProps({} as ProviderProps);

  const keys = getObjectKeys(customContextProviderProps);

  const customContextHooks = keys.reduce((hooks, key) => {
    const name = createCustomContextHookName(String(key));
    const useCustomContextHook = createCustomContextHook(Context, key);

    return {
      ...hooks,
      [name]: useCustomContextHook,
    };
  }, {} as CustomContextHooks<ProviderProps>);

  return customContextHooks;
};

export default createCustomContextHooks;
