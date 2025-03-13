import { useCallback } from 'react';

export const useHandleOnClick = () => {
  const handleOnClick = useCallback(() => {
    // TODO: Trigger upgrade abort
  }, []);

  return handleOnClick;
};
