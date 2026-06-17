import { messages } from '../../../../shared/gettext';
import { StatusDialog } from '../../status-dialog';

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
}

export default function ClearAccountHistoryDialog({ open, onOpenChange, onConfirm }: Props) {
  return (
    <StatusDialog variant="caution" open={open} onOpenChange={onOpenChange}>
      <StatusDialog.Text>
        {
          // TRANSLATORS: Text that informs the users about consequences of creating a new account.
          messages.pgettext(
            'login-view',
            'You already have a saved account number, by creating a new account the saved account number will be removed from this device. This cannot be undone.',
          )
        }
      </StatusDialog.Text>
      <StatusDialog.Text>
        {
          // TRANSLATORS: Text that asks the user if they really want to create a new account.
          messages.pgettext('login-view', 'Do you want to create a new account?')
        }
      </StatusDialog.Text>
      <StatusDialog.ButtonGroup>
        <StatusDialog.Button onClick={onConfirm}>
          <StatusDialog.Button.Text>
            {
              // TRANSLATORS: Button which confirms the action to create a new account.
              messages.pgettext('login-view', 'Create new account')
            }
          </StatusDialog.Button.Text>
        </StatusDialog.Button>
        <StatusDialog.CloseButton>
          <StatusDialog.CloseButton.Text>
            {messages.gettext('Cancel')}
          </StatusDialog.CloseButton.Text>
        </StatusDialog.CloseButton>
      </StatusDialog.ButtonGroup>
    </StatusDialog>
  );
}
