import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useLocale() {
  const locale = useSelector((state) => state.userInterface.locale);
  const preferredLocale = useSelector((state) => state.settings.guiSettings.preferredLocale);

  const { getPreferredLocaleList, setPreferredLocale: contextSetPreferredLocale } = useAppContext();
  const locales = getPreferredLocaleList();

  const setPreferredLocale = React.useCallback(
    async (value: string) => {
      try {
        await contextSetPreferredLocale(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set preferred locale', message);
      }
    },
    [contextSetPreferredLocale],
  );

  return { locale, preferredLocale, setPreferredLocale, locales };
}
