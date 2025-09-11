import { useCallback } from 'react';

import { ILinuxSplitTunnelingApplication } from '../../../../../../../../shared/application-types';
import { ApplicationList } from '../../../application-list';
import { useFilteredApplications, useLaunchApplication } from '../../hooks';
import { LinuxApplicationRow } from './components';

export function LinuxApplicationList() {
  const launchApplication = useLaunchApplication();

  const rowRenderer = useCallback(
    (application: ILinuxSplitTunnelingApplication) => (
      <LinuxApplicationRow application={application} onSelect={launchApplication} />
    ),
    [launchApplication],
  );

  const filteredApplications = useFilteredApplications();

  return (
    <ApplicationList
      data-testid="linux-applications"
      applications={filteredApplications}
      rowRenderer={rowRenderer}
    />
  );
}
