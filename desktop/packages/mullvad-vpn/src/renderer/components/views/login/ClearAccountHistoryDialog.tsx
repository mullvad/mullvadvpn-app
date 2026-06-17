import { messages } from '../../../../shared/gettext';
import { StatusDialog } from '../../status-dialog';

interface Props {
  open: boolean;
  onConfirm: () => void;
  onOpenChange: (open: boolean) => void;
}

export default function ClearAccountHistoryDialog({ open, onOpenChange, onConfirm }: Props) {
  return (
    <StatusDialog variant="caution" open={open} onOpenChange={onOpenChange}>
      <StatusDialog.Text>
        {
          // TRANSLATORS: Text that informs the user about the consequences of clearing the saved
          // TRANSLATORS: account number.
          messages.pgettext(
            'login-view',
            'Removing the saved account number from this device cannot be undone.',
          )
        }
      </StatusDialog.Text>
      <StatusDialog.Text>
        {
          // TRANSLATORS: Text that asks the user if they really want to remove the saved account.
          messages.pgettext('login-view', 'Do you want to remove the saved account number?')
        }
      </StatusDialog.Text>
      <StatusDialog.ButtonGroup>
        <StatusDialog.Button onClick={onConfirm} variant="destructive">
          <StatusDialog.Button.Text>
            {
              // TRANSLATORS: Button label in confirmation dialog that confirms a remove action.
              messages.gettext('Remove')
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
