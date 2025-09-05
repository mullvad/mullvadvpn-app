import { messages } from '../../../../../../../../../../shared/gettext';
import { Button } from '../../../../../../../../../lib/components';
import { Colors } from '../../../../../../../../../lib/foundations';
import { ModalAlert, ModalAlertType } from '../../../../../../../../Modal';
import { useHeaderContext } from '../../HeaderContext';
import { useHideUnsupportedDialog } from './hooks';

export function UnsupportedDialog() {
  const { showUnsupportedDialog } = useHeaderContext();
  const hideUnsupportedDialog = useHideUnsupportedDialog();
  const iconColor: Colors = 'white';
  const unsupportedMessage = messages.pgettext(
    'split-tunneling-view',
    'To use split tunneling, please change to a Linux kernel version that supports cgroup v1.',
  );

  const buttons = [
    <Button key="cancel" onClick={hideUnsupportedDialog}>
      <Button.Text>{messages.pgettext('split-tunneling-view', 'Got it!')}</Button.Text>
    </Button>,
  ];

  return (
    <ModalAlert
      isOpen={showUnsupportedDialog}
      type={ModalAlertType.warning}
      iconColor={iconColor}
      message={unsupportedMessage}
      buttons={buttons}
      close={hideUnsupportedDialog}
    />
  );
}
