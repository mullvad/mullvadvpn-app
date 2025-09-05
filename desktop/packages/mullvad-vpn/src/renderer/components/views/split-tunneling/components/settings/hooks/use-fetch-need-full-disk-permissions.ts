import { useCallback } from 'react';

import { useAppContext } from '../../../../../../context';
import { useSettingsContext } from '../SettingsContext';

export function useFetchNeedFullDiskPermissions() {
  const { needFullDiskPermissions } = useAppContext();
  const { setLoadingDiskPermissions, setSplitTunnelingAvailable } = useSettingsContext();

  const fetchNeedFullDiskPermissions = useCallback(async () => {
    setLoadingDiskPermissions(true);
    const needPermissions = await needFullDiskPermissions();
    setSplitTunnelingAvailable(!needPermissions);
    setLoadingDiskPermissions(false);
  }, [needFullDiskPermissions, setLoadingDiskPermissions, setSplitTunnelingAvailable]);

  return fetchNeedFullDiskPermissions;
}
