import { messages } from '../../../../../../../../shared/gettext';
import { StatusDialog, StatusDialogProps } from '../../../../../../status-dialog';

type AutomaticEntryInfoDialogProps = Omit<StatusDialogProps, 'children' | 'variant'>;

export function AutomaticEntryInfoDialog({
  open,
  onOpenChange,
  ...props
}: AutomaticEntryInfoDialogProps) {
  return (
    <StatusDialog variant="info" open={open} onOpenChange={onOpenChange} {...props}>
      <StatusDialog.TextGroup>
        <StatusDialog.Text>
          {
            // TRANSLATORS: Text in dialog explaining the behavior of the “Automatic” location option
            messages.pgettext(
              'select-location-view',
              'When the “Automatic” location is selected, the app automatically picks a random entry server, prioritizing those closer to the exit location for better performance.',
            )
          }
        </StatusDialog.Text>
        <StatusDialog.Text>
          {
            // TRANSLATORS: Text in dialog warning that any enabled filters are ignored for the entry server when the “Automatic” location is selected
            messages.pgettext(
              'select-location-view',
              'Attention: With the “Automatic” location, any enabled filters are ignored for the entry server.',
            )
          }
        </StatusDialog.Text>
      </StatusDialog.TextGroup>
      <StatusDialog.ButtonGroup>
        <StatusDialog.CloseButton>
          <StatusDialog.CloseButton.Text>
            {messages.gettext('Got it!')}
          </StatusDialog.CloseButton.Text>
        </StatusDialog.CloseButton>
      </StatusDialog.ButtonGroup>
    </StatusDialog>
  );
}
