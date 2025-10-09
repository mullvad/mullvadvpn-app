import React from 'react';

export type UseQueryProps<T> = {
  enabled?: boolean;
  queryFn: () => Promise<T>;
  queryKey: string[];
};

export const useQuery = <T>({ queryFn, queryKey, enabled = true }: UseQueryProps<T>) => {
  const [data, setData] = React.useState<T | undefined>(undefined);
  const [error, setError] = React.useState<Error | undefined>(undefined);
  const [isError, setIsError] = React.useState<boolean>(false);
  const [isFetching, setIsFetching] = React.useState<boolean>(false);

  const hasLoadedRef = React.useRef(false);
  const mountedRef = React.useRef(false);
  const runIdRef = React.useRef(0);

  const cacheKey = queryKey.join();

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
      if (isActive()) {
        setIsFetching(false);
      }
      if (!hasLoadedRef.current) {
        hasLoadedRef.current = true;
      }
    }
  }, [hasLoadedRef, queryFn]);

  const isLoading = isFetching && !hasLoadedRef.current;

  React.useEffect(() => {
    mountedRef.current = true;
    if (enabled) void run();
    return () => {
      mountedRef.current = false;
    };
  }, [enabled, cacheKey, run]);

  return { data, error, isError, isLoading, isFetching, refetch: run };
};
