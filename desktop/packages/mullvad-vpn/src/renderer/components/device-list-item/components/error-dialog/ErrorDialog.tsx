import { messages } from '../../../../../shared/gettext';
import { Button } from '../../../../lib/components';
import { IModalAlertProps, ModalAlert, ModalAlertType } from '../../../Modal';
import { useDeviceListItemContext } from '../../DeviceListItemContext';

export type ErrorDialogProps = IModalAlertProps;

export function ErrorDialog({ isOpen }: ErrorDialogProps) {
  const { resetError } = useDeviceListItemContext();
  return (
    <ModalAlert
      isOpen={isOpen}
      type={ModalAlertType.failure}
      buttons={[
        <Button key="close" onClick={resetError}>
          <Button.Text>{messages.gettext('Close')}</Button.Text>
        </Button>,
      ]}
      close={resetError}
      message={messages.pgettext('device-management', 'Failed to remove device')}
    />
  );
}
