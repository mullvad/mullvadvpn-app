import type { Context } from 'react';

import type withCustomContextProvider from './with-custom-context-provider';

export type CamelCase<String extends string> = String extends `${infer P1}_${infer P2}${infer P3}`
  ? `${Lowercase<P1>}${Uppercase<P2>}${CamelCase<P3>}`
  : Lowercase<String>;

export type KeysToCamelCase<T> = {
  [K in keyof T as CamelCase<string & K>]: T[K] extends object ? KeysToCamelCase<T[K]> : T[K];
};

export type SnakeCase<String extends string> = String extends `${infer Type}${infer Substring}`
  ? `${Type extends Capitalize<Type> ? '_' : ''}${Lowercase<Type>}${SnakeCase<Substring>}`
  : String;

export type KeysToSnakeCase<Type> = {
  [Key in keyof Type as SnakeCase<string & Key>]: Type[Key];
};

export type CustomContextSetValuesCallback<ContextValues extends CustomContextValues> = (
  stateValues: ContextValues,
) => ContextValues;

export type CustomContextSetValues<ContextValues extends CustomContextValues> = (
  values: Partial<ContextValues> | CustomContextSetValuesCallback<ContextValues>,
) => void;

export type CustomContextValues = object | undefined;

export type CustomContext<ContextValues extends CustomContextValues> = {
  values: ContextValues;
  setValues: CustomContextSetValues<ContextValues>;
};

export type CustomContextReact<ContextValues extends CustomContextValues> = Context<
  CustomContext<ContextValues>
>;

export type UseContext<ContextValues extends CustomContextValues> =
  () => CustomContext<ContextValues>;

export type ForbiddenProps<Props extends object> = {
  [key in keyof Props]: never;
};

export type KeyToHook<Key> = CamelCase<`use_${SnakeCase<Extract<Key, string>>}`>;

export type PropsToHooks<Props extends object> = {
  [Key in keyof Props as KeyToHook<Key>]: Props[Key];
};

export type CustomContextHooks<ContextProviderProps extends object> = {
  [Key in keyof PropsToHooks<ContextProviderProps>]: () => PropsToHooks<ContextProviderProps>[Key];
};

export type CreateContextWithProvider<
  ContextValues extends CustomContextValues,
  ProviderProps extends object,
> = [
  useContext: UseContext<ContextValues>,
  customContextHooks: CustomContextHooks<ProviderProps>,
  withCustomContextProvider: <ComponentProps extends Omit<ComponentProps, keyof ProviderProps>>(
    Component: React.FunctionComponent<ComponentProps>,
  ) => ReturnType<typeof withCustomContextProvider<ComponentProps, ProviderProps>>,
  customContextProvider: React.FunctionComponent<ProviderProps>,
];
