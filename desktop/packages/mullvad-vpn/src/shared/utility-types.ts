export type Async<F extends (...args: unknown[]) => unknown> = (
  ...args: Parameters<F>
) => Promise<ReturnType<F>>;
