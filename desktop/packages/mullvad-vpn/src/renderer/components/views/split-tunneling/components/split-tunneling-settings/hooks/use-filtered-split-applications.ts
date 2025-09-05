import { useMemo } from 'react';

import { useSelector } from '../../../../../../redux/store';
import { includesSearchTerm } from '../../../utils';
import { useSplitTunnelingSettingsContext } from '../SplitTunnelingSettingsContext';

export function useFilteredSplitApplications() {
  const { searchTerm } = useSplitTunnelingSettingsContext();
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
