import { messages } from '../../../../../shared/gettext';
import { StatusDialog, type StatusDialogProps } from '../../../status-dialog';
import { useInfoContext } from '../../InfoContext';

export type InfoDialogProps = Omit<StatusDialogProps, 'variant'>;

function InfoDialog({ children, ...props }: InfoDialogProps) {
  const { open, onOpenChange } = useInfoContext();

  return (
    <StatusDialog variant="info" open={open} onOpenChange={onOpenChange} {...props}>
      {children}
      <StatusDialog.CloseButton>
        <StatusDialog.CloseButton.Text>{messages.gettext('Got it!')}</StatusDialog.CloseButton.Text>
      </StatusDialog.CloseButton>
    </StatusDialog>
  );
}

const InfoDialogNamespace = Object.assign(InfoDialog, {
  Text: StatusDialog.Text,
  Title: StatusDialog.Title,
});

export { InfoDialogNamespace as InfoDialog };
