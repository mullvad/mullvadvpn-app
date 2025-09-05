import { useMemo } from 'react';

import { useSelector } from '../../../../../../redux/store';
import { includesSearchTerm } from '../../../utils';
import { useSettingsContext } from '../SettingsContext';

export function useFilteredSplitApplications() {
  const { searchTerm } = useSettingsContext();
  const splitTunnelingApplications = useSelector(
    (state) => state.settings.splitTunnelingApplications,
  );

  const filteredSplitApplications = useMemo(
    () =>
      splitTunnelingApplications.filter((application) =>
        includesSearchTerm(application, searchTerm),
      ),
    [splitTunnelingApplications, searchTerm],
  );

  return filteredSplitApplications;
}
