import { messages } from '../../../../shared/gettext';
import { CautionDialog } from '../../caution-dialog';

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
}

export default function ClearAccountHistoryDialog({ open, onOpenChange, onConfirm }: Props) {
  return (
    <CautionDialog open={open} onOpenChange={onOpenChange}>
      <CautionDialog.Text>
        {
          // TRANSLATORS: Text that informs the users about consequences of creating a new account.
          messages.pgettext(
            'login-view',
            'You already have a saved account number, by creating a new account the saved account number will be removed from this device. This cannot be undone.',
          )
        }
      </CautionDialog.Text>
      <CautionDialog.Text>
        {
          // TRANSLATORS: Text that asks the user if they really want to create a new account.
          messages.pgettext('login-view', 'Do you want to create a new account?')
        }
      </CautionDialog.Text>
      <CautionDialog.ButtonGroup>
        <CautionDialog.Button onClick={onConfirm}>
          <CautionDialog.Button.Text>
            {
              // TRANSLATORS: Button which confirms the action to create a new account.
              messages.pgettext('login-view', 'Create new account')
            }
          </CautionDialog.Button.Text>
        </CautionDialog.Button>
        <CautionDialog.CloseButton>
          <CautionDialog.CloseButton.Text>
            {messages.gettext('Cancel')}
          </CautionDialog.CloseButton.Text>
        </CautionDialog.CloseButton>
      </CautionDialog.ButtonGroup>
    </CautionDialog>
  );
}
