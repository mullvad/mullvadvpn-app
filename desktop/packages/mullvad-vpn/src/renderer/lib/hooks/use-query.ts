import React from 'react';

import { useEffectEvent } from '../utility-hooks';

export type UseQueryProps<T> = {
  enabled?: boolean;
  queryFn: () => Promise<T | undefined>;
  queryKey: string[];
};

export const useQuery = <T>({ queryFn, queryKey, enabled = true }: UseQueryProps<T>) => {
  const [data, setData] = React.useState<T | undefined>(undefined);
  const [error, setError] = React.useState<Error | undefined>(undefined);
  const [isError, setIsError] = React.useState<boolean>(false);
  const [isFetching, setIsFetching] = React.useState<boolean>(false);

  const [hasLoaded, setHasLoaded] = React.useState(false);
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
    }

    if (isActive()) {
      setIsFetching(false);
    }
    if (!hasLoaded) {
      setHasLoaded(true);
    }
  }, [hasLoaded, queryFn]);

  const isLoading = isFetching && !hasLoaded;

  // TODO: Remove the use of useEffectEvent. This is used as an escape hatch
  // in order to be able to continue setting state from a useEffect without
  // lint errors.
  //
  // The entire logic should be rewritten to no longer depend on setting
  // state from an effect.
  const runQuery = useEffectEvent(() => {
    if (enabled) {
      void run();
    }
  });

  React.useEffect(() => {
    mountedRef.current = true;
    runQuery();
    return () => {
      mountedRef.current = false;
    };
  }, [enabled, cacheKey]);

  return { data, error, isError, isLoading, isFetching, refetch: run };
};
