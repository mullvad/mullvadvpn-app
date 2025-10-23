export type Async<F extends (...args: unknown[]) => unknown> = (
  ...args: Parameters<F>
) => Promise<ReturnType<F>>;

export type ValueOfArray<T extends ReadonlyArray<unknown> | Array<unknown>> = T[number];
