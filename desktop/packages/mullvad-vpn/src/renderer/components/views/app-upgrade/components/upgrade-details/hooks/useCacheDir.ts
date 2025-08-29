import { useEffect, useState } from 'react';

import log from '../../../../../../../shared/logging';
import { useAppContext } from '../../../../../../context';
import { useMounted } from '../../../../../../lib/utility-hooks';

export function useCacheDir() {
  const { getAppUpgradeCacheDir } = useAppContext();
  const [cacheDir, setCacheDir] = useState<string>();
  const isMounted = useMounted();

  useEffect(() => {
    const fetchCacheDir = async () => {
      try {
        const cacheDir = await getAppUpgradeCacheDir();
        if (isMounted()) {
          setCacheDir(cacheDir);
        }
      } catch (e) {
        const error = e as Error;
        log.warn(`Failed to fetch cache dir: ${error.message}`);
      }
    };

    void fetchCacheDir();
  }, [getAppUpgradeCacheDir, isMounted]);

  return cacheDir;
}
