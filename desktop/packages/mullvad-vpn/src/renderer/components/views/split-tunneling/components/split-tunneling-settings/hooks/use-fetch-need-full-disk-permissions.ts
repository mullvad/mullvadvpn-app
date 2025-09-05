import { useCallback } from 'react';

import { useAppContext } from '../../../../../../context';
import { useSplitTunnelingSettingsContext } from '../SplitTunnelingSettingsContext';

export function useFetchNeedFullDiskPermissions() {
  const { needFullDiskPermissions } = useAppContext();
  const { setLoadingDiskPermissions, setSplitTunnelingAvailable } =
    useSplitTunnelingSettingsContext();

  const fetchNeedFullDiskPermissions = useCallback(async () => {
    setLoadingDiskPermissions(true);
    const needPermissions = await needFullDiskPermissions();
    setSplitTunnelingAvailable(!needPermissions);
    setLoadingDiskPermissions(false);
  }, [needFullDiskPermissions, setLoadingDiskPermissions, setSplitTunnelingAvailable]);

  return fetchNeedFullDiskPermissions;
}
