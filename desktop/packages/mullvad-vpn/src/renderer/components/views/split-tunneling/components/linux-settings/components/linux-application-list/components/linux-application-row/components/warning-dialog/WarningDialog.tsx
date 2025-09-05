import { ModalAlert, ModalAlertType } from '../../../../../../../../../../Modal';
import { useDisabled, useWarningColor } from '../../hooks';
import { useLinuxApplicationRowContext } from '../../LinuxApplicationRowContext';
import { CancelButton, LaunchButton } from './components';
import { useHideWarningDialog, useWarningMessage } from './hooks';

export function WarningDialog() {
  const { showWarningDialog } = useLinuxApplicationRowContext();
  const disabled = useDisabled();
  const hideWarningDialog = useHideWarningDialog();
  const warningColor = useWarningColor();
  const warningMessage = useWarningMessage();

  const warningDialogButtons = disabled
    ? [<CancelButton key="cancel" />]
    : [<LaunchButton key="launch" />, <CancelButton key="cancel" />];

  return (
    <ModalAlert
      isOpen={showWarningDialog}
      type={ModalAlertType.warning}
      iconColor={warningColor}
      message={warningMessage}
      buttons={warningDialogButtons}
      close={hideWarningDialog}
    />
  );
}
