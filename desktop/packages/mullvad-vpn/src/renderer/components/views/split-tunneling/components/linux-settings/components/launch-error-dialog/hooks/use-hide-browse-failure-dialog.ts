import { useCallback } from 'react';

import { useLinuxSettingsContext } from '../../../LinuxSettingsContext';

export function useHideBrowseFailureDialog() {
  const { setBrowseError } = useLinuxSettingsContext();

  const hideBrowseFailureDialog = useCallback(() => setBrowseError(undefined), [setBrowseError]);

  return hideBrowseFailureDialog;
}
