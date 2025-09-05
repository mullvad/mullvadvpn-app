import { useMemo } from 'react';

import { includesSearchTerm } from '../../../utils';
import { useLinuxSettingsContext } from '../LinuxSettingsContext';

export function useFilteredApplications() {
  const { applications, searchTerm } = useLinuxSettingsContext();

  const filteredApplications = useMemo(
    () => applications?.filter((application) => includesSearchTerm(application, searchTerm)),
    [applications, searchTerm],
  );

  return filteredApplications;
}
