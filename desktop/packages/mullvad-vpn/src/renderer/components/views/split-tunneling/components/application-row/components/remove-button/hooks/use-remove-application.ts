import { useCallback } from 'react';

import { useApplicationRowContext } from '../../../ApplicationRowContext';

export function useRemoveApplication() {
  const { application, onRemove } = useApplicationRowContext();

  const removeApplication = useCallback(() => {
    onRemove?.(application);
  }, [application, onRemove]);

  return removeApplication;
}
