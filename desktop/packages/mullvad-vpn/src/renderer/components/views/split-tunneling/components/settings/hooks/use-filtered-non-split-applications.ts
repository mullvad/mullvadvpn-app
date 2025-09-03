import { useMemo } from 'react';

import { useSelector } from '../../../../../../redux/store';
import { includesSearchTerm } from '../../../utils';
import { useSettingsContext } from '../SettingsContext';

export function useFilteredNonSplitApplications() {
  const { applications, searchTerm } = useSettingsContext();
  const splitTunnelingApplications = useSelector(
    (state) => state.settings.splitTunnelingApplications,
  );

  const filteredNonSplitApplications = useMemo(() => {
    return applications?.filter(
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
