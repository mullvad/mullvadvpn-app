import { useLinuxSettingsContext } from '../LinuxSettingsContext';
import { useFilteredApplications } from './use-filtered-applications';

export function useShowSpinner() {
  const { searchTerm } = useLinuxSettingsContext();
  const filteredApplications = useFilteredApplications();

  const showNoSearchResult =
    searchTerm === '' && (filteredApplications === undefined || filteredApplications.length === 0);

  return showNoSearchResult;
}
