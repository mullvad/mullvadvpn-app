import { messages } from '../../../../shared/gettext';
import { CautionDialog } from '../../caution-dialog';

interface Props {
  open: boolean;
  onConfirm: () => void;
  onOpenChange: (open: boolean) => void;
}

export default function ClearAccountHistoryDialog({ open, onOpenChange, onConfirm }: Props) {
  return (
    <CautionDialog open={open} onOpenChange={onOpenChange}>
      <CautionDialog.Text>
        {
          // TRANSLATORS: Text that informs the user about the consequences of clearing the saved
          // TRANSLATORS: account number.
          messages.pgettext(
            'login-view',
            'Removing the saved account number from this device cannot be undone.',
          )
        }
      </CautionDialog.Text>
      <CautionDialog.Text>
        {
          // TRANSLATORS: Text that asks the user if they really want to remove the saved account.
          messages.pgettext('login-view', 'Do you want to remove the saved account number?')
        }
      </CautionDialog.Text>
      <CautionDialog.ButtonGroup>
        <CautionDialog.Button onClick={onConfirm} variant="destructive">
          <CautionDialog.Button.Text>
            {
              // TRANSLATORS: Button label in confirmation dialog that confirms a remove action.
              messages.gettext('Remove')
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
