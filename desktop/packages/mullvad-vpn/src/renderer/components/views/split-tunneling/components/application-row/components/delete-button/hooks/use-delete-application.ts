import { useCallback } from 'react';

import { useApplicationRowContext } from '../../../ApplicationRowContext';

export function useDeleteApplication() {
  const { application, onDelete } = useApplicationRowContext();

  const deleteApplication = useCallback(() => {
    onDelete?.(application);
  }, [application, onDelete]);

  return deleteApplication;
}
