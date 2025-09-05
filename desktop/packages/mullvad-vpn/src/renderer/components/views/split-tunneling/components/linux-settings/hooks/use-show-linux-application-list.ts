import { useFilteredApplications } from './use-filtered-applications';

export function useShowLinuxApplicationList() {
  const filteredApplications = useFilteredApplications();

  const showLinuxApplicationList =
    filteredApplications !== undefined && filteredApplications.length > 0;

  return showLinuxApplicationList;
}
