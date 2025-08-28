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
        <Button key="confirm" onClick={props.onConfirm}>
          <Button.Text>
            {
              // TRANSLATORS: Button which confirms the action to create a new account.
              messages.pgettext('login-view', 'Create new account')
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
          // TRANSLATORS: Text that informs the users about consequences of creating a new account.
          messages.pgettext(
            'login-view',
            'You already have a saved account number, by creating a new account the saved account number will be removed from this device. This cannot be undone.',
          )
        }
      </ModalMessage>
      <ModalMessage>
        {
          // TRANSLATORS: Text that asks the user if they really want to create a new account.
          messages.pgettext('login-view', 'Do you want to create a new account?')
        }
      </ModalMessage>
    </ModalAlert>
  );
}
