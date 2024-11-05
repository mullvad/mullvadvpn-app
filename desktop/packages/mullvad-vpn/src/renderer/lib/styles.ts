type NonTransientPropKey<K> = K extends `$${infer L}` ? L : K;

export type NonTransientProps<T, K extends NonTransientPropKey<keyof T>> = {
  [P in keyof T as NonTransientPropKey<P> extends K ? NonTransientPropKey<P> : P]: T[P];
};

export type TransientProps<T, K extends keyof T> = {
  [P in keyof T as P extends K ? `$${P & string}` : P]: T[P];
};
