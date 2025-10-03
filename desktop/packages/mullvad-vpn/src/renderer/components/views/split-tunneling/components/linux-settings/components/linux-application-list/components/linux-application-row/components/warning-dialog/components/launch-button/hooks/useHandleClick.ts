import { useCallback } from 'react';

import { useLinuxApplicationRowContext } from '../../../../../LinuxApplicationRowContext';

export function useHandleClick() {
  const { application, onSelect, setShowWarningDialog } = useLinuxApplicationRowContext();
  const handleClick = useCallback(() => {
    setShowWarningDialog(false);
    onSelect?.(application);
  }, [application, onSelect, setShowWarningDialog]);

  return handleClick;
}
