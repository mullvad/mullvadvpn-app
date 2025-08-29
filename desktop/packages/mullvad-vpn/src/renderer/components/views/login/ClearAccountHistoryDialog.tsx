import { messages } from '../../../../shared/gettext';
import { Button } from '../../../lib/components';
import { ModalAlert, ModalAlertType, ModalMessage } from '../../Modal';

interface Props {
  visible: boolean;
  onConfirm: () => void;
  onHide: () => void;
}

export default function ClearAccountHistoryDialog(props: Props) {
  return (
    <ModalAlert
      isOpen={props.visible}
      type={ModalAlertType.caution}
      buttons={[
        <Button variant="destructive" key="confirm" onClick={props.onConfirm}>
          <Button.Text>
            {
              // TRANSLATORS: Button label in confirmation dialog that confirms a remove action.
              messages.gettext('Remove')
            }
          </Button.Text>
        </Button>,
        <Button key="back" onClick={props.onHide}>
          <Button.Text>{messages.gettext('Cancel')}</Button.Text>
        </Button>,
      ]}
      close={props.onHide}>
      <ModalMessage>
        {
          // TRANSLATORS: Text that informs the user about the consequences of clearing the saved
          // TRANSLATORS: account number.
          messages.pgettext(
            'login-view',
            'Removing the saved account number from this device cannot be undone.',
          )
        }
      </ModalMessage>
      <ModalMessage>
        {
          // TRANSLATORS: Text that asks the user if they really want to remove the saved account.
          messages.pgettext('login-view', 'Do you want to remove the saved account number?')
        }
      </ModalMessage>
    </ModalAlert>
  );
}
