import { Dialog } from '../../../../../../../../../../../lib/components/dialog';
import { useDisabled, useWarningColor } from '../../hooks';
import { useLinuxApplicationRowContext } from '../../LinuxApplicationRowContext';
import { CancelButton, LaunchButton } from './components';
import { useWarningMessage } from './hooks';

export function WarningDialog() {
  const { showWarningDialog, setShowWarningDialog } = useLinuxApplicationRowContext();
  const disabled = useDisabled();
  const warningColor = useWarningColor();
  const warningMessage = useWarningMessage();

  const warningDialogButtons = disabled
    ? [<CancelButton key="cancel" />]
    : [<LaunchButton key="launch" />, <CancelButton key="cancel" />];

  return (
    <Dialog open={showWarningDialog} onOpenChange={setShowWarningDialog}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <Dialog.Icon icon="info-circle" color={warningColor} />
            <Dialog.Text>{warningMessage}</Dialog.Text>
            <Dialog.ButtonGroup>{warningDialogButtons}</Dialog.ButtonGroup>
          </Dialog.PopupContent>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
}
