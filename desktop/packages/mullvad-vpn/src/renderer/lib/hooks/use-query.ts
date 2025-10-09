import React from 'react';

export type UseQueryProps<T> = {
  enabled?: boolean;
  queryFn: () => Promise<T>;
  deps: React.DependencyList;
};

export const useQuery = <T>({ queryFn, enabled = true, deps }: UseQueryProps<T>) => {
  const [data, setData] = React.useState<T | undefined>(undefined);
  const [error, setError] = React.useState<Error | undefined>(undefined);
  const [isError, setIsError] = React.useState<boolean>(false);
  const [isFetching, setIsFetching] = React.useState<boolean>(false);
  const [hasSettledFirst, setHasSettledFirst] = React.useState<boolean>(false);

  const mountedRef = React.useRef(false);
  const runIdRef = React.useRef(0);

  const run = React.useCallback(async () => {
    const runId = ++runIdRef.current;

    const isActive = () => mountedRef.current && runId === runIdRef.current;

    setIsFetching(true);
    setIsError(false);
    setError(undefined);

    try {
      const result = await queryFn();
      if (isActive()) {
        setData(result);
      }
    } catch (err) {
      if (isActive()) {
        setIsError(true);
        setError(err as Error);
      }
    } finally {
      if (isActive()) setIsFetching(false);
      if (!hasSettledFirst) setHasSettledFirst(true);
    }
  }, [hasSettledFirst, queryFn]);

  const isLoading = isFetching && !hasSettledFirst;

  React.useEffect(() => {
    mountedRef.current = true;
    if (enabled) void run();
    return () => {
      mountedRef.current = false;
    };
    // Excluding run from deps to avoid re-fetching on every render
    // eslint-disable-next-line react-compiler/react-compiler
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [enabled, ...deps]);

  return { data, error, isError, isLoading, isFetching, refetch: run };
};
