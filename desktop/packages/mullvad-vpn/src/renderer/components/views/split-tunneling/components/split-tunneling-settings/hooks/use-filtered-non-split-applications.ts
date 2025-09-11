import { useMemo } from 'react';

import { useSelector } from '../../../../../../redux/store';
import { includesSearchTerm } from '../../../utils';
import { useSplitTunnelingSettingsContext } from '../SplitTunnelingSettingsContext';

export function useFilteredNonSplitApplications() {
  const { applications, searchTerm } = useSplitTunnelingSettingsContext();
  const splitTunnelingApplications = useSelector(
    (state) => state.settings.splitTunnelingApplications,
  );

  const filteredNonSplitApplications = useMemo(() => {
    if (!applications) {
      return [];
    }

    return applications.filter(
      (application) =>
        includesSearchTerm(application, searchTerm) &&
        !splitTunnelingApplications.some(
          (splitTunnelingApplication) =>
            application.absolutepath === splitTunnelingApplication.absolutepath,
        ),
    );
  }, [applications, splitTunnelingApplications, searchTerm]);

  return filteredNonSplitApplications;
}
