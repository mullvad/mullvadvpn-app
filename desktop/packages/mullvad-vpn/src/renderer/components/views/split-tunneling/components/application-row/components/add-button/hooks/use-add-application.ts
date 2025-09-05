import { useCallback } from 'react';

import { useApplicationRowContext } from '../../../ApplicationRowContext';

export function useAddApplication() {
  const { application, onAdd } = useApplicationRowContext();

  const addApplication = useCallback(() => {
    onAdd?.(application);
  }, [application, onAdd]);

  return addApplication;
}
