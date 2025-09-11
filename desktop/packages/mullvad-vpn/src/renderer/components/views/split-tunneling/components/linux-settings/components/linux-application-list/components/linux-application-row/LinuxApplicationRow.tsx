import { type ILinuxSplitTunnelingApplication } from '../../../../../../../../../../shared/application-types';
import { ApplicationIcon } from '../../../../../application-icon';
import { ApplicationLabel } from '../../../../../application-label';
import { LaunchButton, WarningDialog, WarningIcon } from './components';
import { useApplication, useDisabled, useHasApplicationWarning } from './hooks';
import { LinuxApplicationRowContextProvider } from './LinuxApplicationRowContext';

export type LinuxApplicationRowProps = {
  application: ILinuxSplitTunnelingApplication;
  onSelect?: (application: ILinuxSplitTunnelingApplication) => void;
};

function LinuxApplicationRowInner() {
  const application = useApplication();
  const disabled = useDisabled();
  const hasApplicationWarning = useHasApplicationWarning();

  return (
    <>
      <LaunchButton>
        <ApplicationIcon icon={application.icon} disabled={disabled} />
        <ApplicationLabel disabled={disabled}>{application.name}</ApplicationLabel>
        {hasApplicationWarning && <WarningIcon />}
      </LaunchButton>
      <WarningDialog />
    </>
  );
}

export function LinuxApplicationRow(props: LinuxApplicationRowProps) {
  return (
    <LinuxApplicationRowContextProvider {...props}>
      <LinuxApplicationRowInner />
    </LinuxApplicationRowContextProvider>
  );
}
